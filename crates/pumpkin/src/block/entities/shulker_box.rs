use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{array::from_fn, sync::Arc};

use crate::block::entities::BlockEntity;
use crate::block::viewer::{ViewerCountListener, ViewerCountTracker, ViewerCountTrackerExt};
use crate::world::World;
use pumpkin_world::inventory::{Clearable, Inventory, sync_write_items_to_nbt};

pub struct ShulkerBoxBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,

    // Viewer
    pub viewers: ViewerCountTracker,
}

impl BlockEntity for ShulkerBoxBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut shulker_box = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        };

        pumpkin_world::inventory::sync_read_items_from_nbt(
            nbt,
            shulker_box
                .items
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );

        shulker_box
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_inventory_nbt(nbt, true);
    }

    fn tick(&self, world: &Arc<World>) {
        self.viewers
            .update_viewer_count::<Self>(self, world, &self.position);
    }

    fn on_block_replaced(self: Arc<Self>, _world: &Arc<World>, _position: &BlockPos) {
        // Shulker boxes retain items when broken
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ViewerCountListener for ShulkerBoxBlockEntity {
    fn on_container_open(&self, world: &Arc<World>, position: &BlockPos) {
        Self::play_sound(world, position, 1);
        // TODO: this.world.emitGameEvent(player, GameEvent.CONTAINER_OPEN, this.pos);
    }

    fn on_container_close(&self, world: &Arc<World>, position: &BlockPos) {
        Self::play_sound(world, position, 0);
        // TODO: this.world.emitGameEvent(player, GameEvent.CONTAINER_CLOSE, this.pos);
    }

    fn on_viewer_count_update(&self, world: &Arc<World>, position: &BlockPos, _old: u16, new: u16) {
        world.add_synced_block_event(*position, Self::OPEN_ANIMATION_EVENT_TYPE, new as u8);
    }
}

impl ShulkerBoxBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:shulker_box";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        }
    }

    pub fn update_viewers(&self, world: &Arc<World>) {
        let viewer_count = self.viewers.current.load(Ordering::Relaxed);
        Self::play_sound(world, &self.position, i32::from(viewer_count));
    }

    fn play_sound(world: &World, position: &BlockPos, viewer_count: i32) {
        let sound = if viewer_count > 0 {
            Sound::BlockShulkerBoxOpen
        } else {
            Sound::BlockShulkerBoxClose
        };

        world.play_sound(sound, SoundCategory::Blocks, &position.to_f64());
    }
}

impl Inventory for ShulkerBoxBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot].clone()
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let res = if !items[slot].is_empty() && amount > 0 {
            items[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot] = stack;
        self.mark_dirty();
    }

    fn on_open(&self) {
        self.viewers.open_container();
    }

    fn on_close(&self) {
        self.viewers.close_container();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for ShulkerBoxBlockEntity {
    fn clear(&self) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}
