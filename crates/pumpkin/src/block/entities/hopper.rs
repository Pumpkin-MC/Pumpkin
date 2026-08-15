use crate::block::entities::BlockEntity;
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, FacingHopper, HopperLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, sync_write_items_to_nbt};
use std::any::Any;
use std::array::from_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};

pub struct HopperBlockEntity {
    pub position: BlockPos,
    pub items: tokio::sync::RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub facing: FacingHopper,
    pub cooldown_time: AtomicI32,
    pub ticked_game_time: AtomicI64,
}

#[must_use]
pub fn to_offset(facing: &FacingHopper) -> Vector3<i32> {
    match facing {
        FacingHopper::Down => (0, -1, 0),
        FacingHopper::North => (0, 0, -1),
        FacingHopper::South => (0, 0, 1),
        FacingHopper::West => (-1, 0, 0),
        FacingHopper::East => (1, 0, 0),
    }
    .into()
}

impl BlockEntity for HopperBlockEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put(
                "TransferCooldown",
                NbtTag::Int(self.cooldown_time.load(Ordering::Relaxed)),
            );
            self.write_inventory_nbt(nbt, true).await;
        })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut hopper = Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            facing: FacingHopper::Down,
            cooldown_time: AtomicI32::from(nbt.get_int("TransferCooldown").unwrap_or(-1)),
            ticked_game_time: AtomicI64::new(0),
        };

        pumpkin_world::inventory::sync_read_items_from_nbt(nbt, hopper.items.get_mut());

        hopper
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.ticked_game_time
                .store(world.get_world_age().await, Ordering::Relaxed);
            if self.cooldown_time.fetch_sub(1, Ordering::Relaxed) <= 0 {
                self.cooldown_time.store(0, Ordering::Relaxed);
                let state = HopperLikeProperties::from_state_id(
                    world.get_block_state(&self.position).id,
                    &Block::HOPPER,
                );
                self.try_move_items(&state, world).await;
            }
        })
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
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put(
            "TransferCooldown",
            NbtTag::Int(self.cooldown_time.load(Ordering::Relaxed)),
        );
        let items = futures::executor::block_on(self.items.read());
        sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HopperBlockEntity {
    pub const INVENTORY_SIZE: usize = 5;
    pub const ID: &'static str = "minecraft:hopper";

    #[must_use]
    pub fn new(position: BlockPos, facing: FacingHopper) -> Self {
        Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            facing,
            cooldown_time: AtomicI32::new(-1),
            ticked_game_time: AtomicI64::new(0),
        }
    }
    async fn try_move_items(&self, state: &HopperLikeProperties, world: &Arc<World>) {
        if self.cooldown_time.load(Ordering::Relaxed) <= 0 && state.enabled {
            let mut success = if self.is_empty().await {
                false
            } else {
                self.eject_items(world).await
            };
            if !self.inventory_full().await {
                success |= self.suck_in_items(world).await;
            }
            if success {
                self.cooldown_time.store(8, Ordering::Relaxed);
                self.mark_dirty();
            }
        }
    }

    async fn inventory_full(&self) -> bool {
        let items = self.items.read().await;
        for item in items.iter() {
            if item.is_empty() || item.item_count != item.get_max_stack_size() {
                return false;
            }
        }
        true
    }

    async fn suck_in_items(&self, world: &Arc<World>) -> bool {
        // TODO getEntityContainer
        let pos_up = &self.position.up();
        let mut search_event = crate::plugin::api::events::inventory::hopper_inventory_search::HopperInventorySearchEvent::new(
            self.position,
            *pos_up,
        );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire(&server, &mut search_event).await;
        }
        if search_event.cancelled {
            return false;
        }

        if let Some(entity) = world.get_block_entity(pos_up)
            && let Some(container) = entity.clone().get_inventory()
        {
            // TODO check WorldlyContainer
            for i in 0..container.size() {
                let mut item = container.get_stack(i).await;
                if !item.is_empty() && container.can_transfer_to(self, i, &item) {
                    //TODO WorldlyContainer
                    let _backup = item.clone();
                    let one_item = item.split(1);
                    if Self::add_one_item(container.as_ref(), self, one_item).await {
                        container.set_stack(i, item).await;
                        // If extracting from furnace output slot (index 2), drop XP as orbs
                        let furnace_output_slot: usize = 2;
                        if i == furnace_output_slot
                            && let Some(experience_container) =
                                entity.clone().to_experience_container()
                        {
                            let xp = experience_container.extract_experience();
                            if xp > 0 {
                                let pos = self.position.to_f64();
                                ExperienceOrbEntity::spawn(world, pos, xp as u32).await;
                            }
                        }
                        return true;
                    }
                }
            }
            return false;
        }
        let (block, state) = world.get_block_and_state(pos_up);
        if !(state.is_solid() && block.has_tag(&tag::Block::MINECRAFT_DOES_NOT_BLOCK_HOPPERS)) {
            let pos_up_f = pos_up.to_f64();
            let search_box = pumpkin_util::math::boundingbox::BoundingBox::new(
                pos_up_f,
                pos_up_f.add_raw(1.0, 1.0, 1.0),
            );
            let entities = world.get_entities_at_box(&search_box);
            for entity_base in entities {
                if let Some(item_entity) = entity_base.clone().get_item_entity() {
                    let mut stack = item_entity.get_item_stack().lock().await;
                    if !stack.is_empty() {
                        let mut pickup_event = crate::plugin::api::events::inventory::inventory_pickup_item::InventoryPickupItemEvent::new(
                            self.position,
                            item_entity.get_entity().entity_id,
                            stack.item.registry_key.to_string(),
                        );
                        if let Some(server) = world.server.upgrade() {
                            server.plugin_manager.fire(&server, &mut pickup_event).await;
                        }
                        if pickup_event.cancelled {
                            continue;
                        }
                        let backup = stack.clone();
                        let one_item = stack.split(1);
                        if Self::add_one_item(self, self, one_item).await {
                            if stack.is_empty() {
                                item_entity.get_entity().remove().await;
                            }
                            return true;
                        }
                        *stack = backup;
                    }
                }
            }
        }
        false
    }

    async fn eject_items(&self, world: &Arc<World>) -> bool {
        // TODO getEntityContainer

        if let Some(entity) = world.get_block_entity(&self.position.offset(to_offset(&self.facing)))
            && let Some(container) = entity.get_inventory()
        {
            // TODO check WorldlyContainer
            let mut is_full = true;
            for i in 0..container.size() {
                let item = container.get_stack(i).await;
                if item.item_count < item.get_max_stack_size() {
                    is_full = false;
                    break;
                }
            }
            if is_full {
                return false;
            }
            let target_pos = self.position.offset(to_offset(&self.facing));
            let items = self.items.read().await;
            for item in items.iter() {
                if !item.is_empty() {
                    let mut move_event = crate::plugin::api::events::inventory::inventory_move_item::InventoryMoveItemEvent::new(
                        self.position,
                        target_pos,
                        item.item.registry_key.to_string(),
                        1,
                    );
                    if let Some(server) = world.server.upgrade() {
                        server.plugin_manager.fire(&server, &mut move_event).await;
                    }
                    if move_event.cancelled {
                        continue;
                    }
                    let mut item_clone = item.clone();
                    let one_item = item_clone.split(1);
                    if Self::add_one_item(self, container.as_ref(), one_item).await {
                        return true;
                    }
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
                let mut dst = to.get_stack(j).await;
                if dst.is_empty() {
                    dst = item.clone();
                    to.set_stack(j, dst).await;
                    success = true;
                } else if dst.item_count < dst.get_max_stack_size() && dst.item == item.item {
                    // TODO check Components equal
                    dst.item_count += 1;
                    to.set_stack(j, dst).await;
                    success = true;
                }
                if success {
                    if to_empty
                        && let Some(hopper) = to.as_any().downcast_ref::<Self>()
                        && hopper.cooldown_time.load(Ordering::Relaxed) <= 8
                    {
                        if let Some(from_hopper) = from.as_any().downcast_ref::<Self>() {
                            if from_hopper.cooldown_time.load(Ordering::Relaxed)
                                >= hopper.cooldown_time.load(Ordering::Relaxed)
                            {
                                hopper.cooldown_time.store(7, Ordering::Relaxed);
                            } else {
                                hopper.cooldown_time.store(8, Ordering::Relaxed);
                            }
                        } else {
                            hopper.cooldown_time.store(8, Ordering::Relaxed);
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

impl Inventory for HopperBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            let items = self.items.read().await;
            items.iter().all(ItemStack::is_empty)
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let items = self.items.read().await;
            items[slot].clone()
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
            self.mark_dirty();
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            let res = if !items[slot].is_empty() && amount > 0 {
                items[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            };
            self.mark_dirty();
            res
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items[slot] = stack;
            self.mark_dirty();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for HopperBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.mark_dirty();
        })
    }
}
