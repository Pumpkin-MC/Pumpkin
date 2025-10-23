use crate::BlockStateId;
use crate::block::entities::BlockEntity;
use crate::inventory::{Clearable, Inventory, split_stack};
use crate::item::ItemStack;
use crate::world::SimpleWorld;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, HopperFacing, HopperLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::any::Any;
use std::array::from_fn;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct HopperBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,
    pub facing: HopperFacing,
    pub cooldown_time: AtomicI32,
    pub ticked_game_time: AtomicI64,
}

pub fn to_offset(facing: &HopperFacing) -> Vector3<i32> {
    match facing {
        HopperFacing::Down => (0, -1, 0),
        HopperFacing::North => (0, 0, -1),
        HopperFacing::South => (0, 0, 1),
        HopperFacing::West => (-1, 0, 0),
        HopperFacing::East => (1, 0, 0),
    }
    .into()
}

#[async_trait]
impl BlockEntity for HopperBlockEntity {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        nbt.put(
            "TransferCooldown",
            NbtTag::Int(
                self.cooldown_time
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
        );
        // Safety precaution
        //self.clear().await;
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let hopper = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            facing: HopperFacing::Down,
            cooldown_time: AtomicI32::from(nbt.get_int("TransferCooldown").unwrap_or(-1)),
            ticked_game_time: AtomicI64::new(0),
        };

        hopper.read_data(nbt, &hopper.items);

        hopper
    }

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        self.ticked_game_time.store(
            world.get_world_age().await,
            std::sync::atomic::Ordering::Relaxed,
        );
        if self
            .cooldown_time
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
            <= 0
        {
            self.cooldown_time
                .store(0, std::sync::atomic::Ordering::Relaxed);
            let state = HopperLikeProperties::from_state_id(
                world.get_block_state(&self.position).await.id,
                &Block::HOPPER,
            );
            self.try_move_items(&state, &world).await;
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn set_block_state(&mut self, block_state: BlockStateId) {
        // TODO !!!IMPORTANT!!! set block state when loading the chunk
        self.facing = HopperLikeProperties::from_state_id(block_state, &Block::HOPPER).facing;
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_block_entity(&self) -> &dyn BlockEntity {
        self
    }
}

impl HopperBlockEntity {
    pub const INVENTORY_SIZE: usize = 5;
    pub const ID: &'static str = "minecraft:hopper";

    pub fn new(position: BlockPos, facing: HopperFacing) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            facing,
            cooldown_time: AtomicI32::new(-1),
            ticked_game_time: AtomicI64::new(0),
        }
    }
    async fn try_move_items(&self, state: &HopperLikeProperties, world: &Arc<dyn SimpleWorld>) {
        if self
            .cooldown_time
            .load(std::sync::atomic::Ordering::Relaxed)
            <= 0
            && state.enabled
        {
            let mut success = false;
            if !self.is_empty().await {
                success = self.eject_items(world).await;
            }
            if !self.inventory_full().await {
                success |= self.suck_in_items(world).await;
            }
            if success {
                self.cooldown_time
                    .store(8, std::sync::atomic::Ordering::Relaxed);
                self.mark_dirty();
            }
        }
    }

    async fn inventory_full(&self) -> bool {
        for i in &self.items {
            let item = i.lock().await;
            if item.is_empty() || item.item_count != item.get_max_stack_size() {
                return false;
            }
        }
        true
    }

    async fn suck_in_items(&self, world: &Arc<dyn SimpleWorld>) -> bool {
        // TODO getEntityContainer
        let pos_up = &self.position.up();
        if let Some(entity) = world.get_block_entity(pos_up).await
            && let Some(container) = entity.get_inventory()
        {
            // TODO check WorldlyContainer
            for i in 0..container.size() {
                let bind = container.get_stack(i).await;
                let mut item = bind.lock().await;
                if !item.is_empty() && container.can_transfer_to(self, i, &item) {
                    //TODO WorldlyContainer
                    let backup = item.clone();
                    let one_item = item.split(1);
                    if Self::add_one_item(container.as_ref(), self, one_item).await {
                        return true;
                    }
                    *item = backup;
                }
            }
            return false;
        }
        let (block, state) = world.get_block_and_state(pos_up).await;
        if !(state.is_solid() && block.has_tag(&tag::Block::MINECRAFT_DOES_NOT_BLOCK_HOPPERS)) {
            // TODO getItemsAtAndAbove(level, hopper)
            return false;
        }
        false
    }

    async fn eject_items(&self, world: &Arc<dyn SimpleWorld>) -> bool {
        // TODO getEntityContainer

        if let Some(entity) = world
            .get_block_entity(&self.position.offset(to_offset(&self.facing)))
            .await
            && let Some(container) = entity.get_inventory()
        {
            // TODO check WorldlyContainer
            let mut is_full = true;
            for i in 0..container.size() {
                let bind = container.get_stack(i).await;
                let item = bind.lock().await;
                if item.item_count < item.get_max_stack_size() {
                    is_full = false;
                    break;
                }
            }
            if is_full {
                return false;
            }
            for i in &self.items {
                let mut item = i.lock().await;
                if !item.is_empty() {
                    //TODO WorldlyContainer
                    let backup = item.clone();
                    let one_item = item.split(1);
                    if Self::add_one_item(self, container.as_ref(), one_item).await {
                        return true;
                    }
                    *item = backup;
                }
            }
        }
        false
    }
    pub async fn add_one_item(from: &dyn Inventory, to: &dyn Inventory, item: ItemStack) -> bool {
        let mut success = false;
        let to_empty = to.is_empty().await;
        for j in 0..to.size() {
            if to.is_valid_slot_for(j, &item) {
                let bind = to.get_stack(j).await;
                let mut dst = bind.lock().await;
                if dst.is_empty() {
                    *dst = item.clone();
                    success = true;
                } else if dst.item_count < dst.get_max_stack_size() && dst.item == item.item {
                    // TODO check Components equal
                    dst.item_count += 1;
                    success = true;
                }
                if success {
                    if to_empty
                        && let Some(hopper) = to.as_any().downcast_ref::<HopperBlockEntity>()
                        && hopper
                            .cooldown_time
                            .load(std::sync::atomic::Ordering::Relaxed)
                            <= 8
                    {
                        if let Some(from_hopper) = from.as_any().downcast_ref::<HopperBlockEntity>()
                        {
                            if from_hopper
                                .cooldown_time
                                .load(std::sync::atomic::Ordering::Relaxed)
                                >= hopper
                                    .cooldown_time
                                    .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                hopper
                                    .cooldown_time
                                    .store(7, std::sync::atomic::Ordering::Relaxed);
                            } else {
                                hopper
                                    .cooldown_time
                                    .store(8, std::sync::atomic::Ordering::Relaxed);
                            }
                        } else {
                            hopper
                                .cooldown_time
                                .store(8, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    to.mark_dirty();
                    return true;
                }
            }
        }
        false
    }
}

#[async_trait]
impl Inventory for HopperBlockEntity {
    fn size(&self) -> usize {
        self.items.len()
    }

    async fn is_empty(&self) -> bool {
        for slot in self.items.iter() {
            if !slot.lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        self.items[slot].clone()
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut removed = ItemStack::EMPTY.clone();
        let mut guard = self.items[slot].lock().await;
        std::mem::swap(&mut removed, &mut *guard);
        removed
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        split_stack(&self.items, slot, amount).await
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        *self.items[slot].lock().await = stack;
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for HopperBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
