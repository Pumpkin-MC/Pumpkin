use pumpkin_data::block_properties::BarrelLikeProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, FacingExt, item_stack::ItemStack};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::sync::RwLock;
use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::block::viewer::{ViewerCountListener, ViewerCountTracker, ViewerCountTrackerExt};
use crate::world::{BlockFlags, World};
use pumpkin_inventory::{Clearable, Inventory, sync_write_items_to_nbt};

use super::BlockEntity;

pub struct BarrelBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
    pub loot_table: std::sync::Mutex<Option<String>>,
    pub loot_table_seed: i64,
}

impl BlockEntity for BarrelBlockEntity {
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
        let loot_table_key = nbt.get_string("LootTable").map(|s| s.to_string());
        let loot_table_seed = nbt.get_long("LootTableSeed").unwrap_or(0);

        let mut barrel = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
            loot_table: std::sync::Mutex::new(loot_table_key),
            loot_table_seed,
        };

        let has_loot_table = barrel
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();
        if !has_loot_table {
            pumpkin_inventory::sync_read_items_from_nbt(
                nbt,
                barrel
                    .items
                    .get_mut()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }

        barrel
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(ref key) = *self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            nbt.put_string("LootTable", key.clone());
            if self.loot_table_seed != 0 {
                nbt.put_long("LootTableSeed", self.loot_table_seed);
            }
        } else {
            self.write_inventory_nbt(nbt, true);
        }
    }

    fn tick(&self, world: &Arc<World>) {
        self.viewers
            .update_viewer_count::<Self>(self, world, &self.position);
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let has_loot_table = self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();
        let mut nbt = NbtCompound::new();
        if !has_loot_table && let Ok(guard) = self.items.try_read() {
            sync_write_items_to_nbt(&*guard, &mut nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn take_loot_table(&self) -> Option<(String, i64)> {
        let mut guard = self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.take().map(|key| (key, self.loot_table_seed))
    }

    fn has_loot_table(&self) -> bool {
        self.loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }
}

impl ViewerCountListener for BarrelBlockEntity {
    fn on_container_open(&self, world: &Arc<World>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockBarrelOpen);
        self.set_open(world, true);
    }

    fn on_container_close(&self, world: &Arc<World>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockBarrelClose);
        self.set_open(world, false);
    }
}

impl BarrelBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const ID: &'static str = "minecraft:barrel";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
            loot_table: std::sync::Mutex::new(None),
            loot_table_seed: 0,
        }
    }

    fn set_open(&self, world: &Arc<World>, open: bool) {
        let state = world.get_block_state(&self.position);
        let mut properties = BarrelLikeProperties::from_state_id(state.id);

        properties.open = open;

        world.set_block_state(
            &self.position,
            properties.to_state_id(&Block::BARREL),
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn play_sound(&self, world: &Arc<World>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        let state = world.get_block_state(&self.position);
        let properties = BarrelLikeProperties::from_state_id(state.id);
        let direction = properties.facing.to_block_direction().to_offset();
        let position = Vector3::new(
            self.position.0.x as f64 + 0.5 + direction.x as f64 / 2.0,
            self.position.0.y as f64 + 0.5 + direction.y as f64 / 2.0,
            self.position.0.z as f64 + 0.5 + direction.z as f64 / 2.0,
        );
        world.play_sound_fine(
            sound,
            SoundCategory::Blocks,
            &position,
            0.5,
            rng.next_f32() * 0.1 + 0.9,
        );
    }
}

impl Inventory for BarrelBlockEntity {
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
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for BarrelBlockEntity {
    fn clear(&self) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}
