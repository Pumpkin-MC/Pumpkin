use pumpkin_data::data_component_impl::IDSetContent;
use pumpkin_data::tag::Taggable;
use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;

use crate::block::entities::BlockEntity;
use crate::world::World;
use pumpkin_world::inventory::{Clearable, Inventory};

pub struct BeaconBlockEntity {
    pub position: BlockPos,
    pub primary_effect: AtomicI32,
    pub secondary_effect: AtomicI32,
    pub levels: AtomicI32,
    pub dirty: AtomicBool,
    pub payment: Arc<Mutex<ItemStack>>,

    // Vanilla Parity Fields
    pub custom_name: Mutex<Option<String>>,
    pub lock_key: Mutex<Option<String>>,
    pub last_check_y: AtomicI32,
}

impl BeaconBlockEntity {
    pub const ID: &'static str = "minecraft:beacon";

    // ContainerData Property Constants
    pub const DATA_LEVELS: usize = 0;
    pub const DATA_PRIMARY: usize = 1;
    pub const DATA_SECONDARY: usize = 2;
    pub const NUM_DATA_VALUES: usize = 3;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            primary_effect: AtomicI32::new(-1),
            secondary_effect: AtomicI32::new(-1),
            levels: AtomicI32::new(0),
            dirty: AtomicBool::new(false),
            payment: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            custom_name: Mutex::new(None),
            lock_key: Mutex::new(None),
            last_check_y: AtomicI32::new(position.0.y - 1),
        }
    }

    /// Replicates the Java `ContainerData` used to sync values to the `BeaconMenu`
    pub fn get_data(&self, id: usize) -> i32 {
        match id {
            Self::DATA_LEVELS => self.levels.load(Ordering::Relaxed),
            Self::DATA_PRIMARY => self.primary_effect.load(Ordering::Relaxed),
            Self::DATA_SECONDARY => self.secondary_effect.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    pub fn set_data(&self, id: usize, value: i32) {
        match id {
            Self::DATA_LEVELS => self.levels.store(value, Ordering::Relaxed),
            Self::DATA_PRIMARY => self.primary_effect.store(value, Ordering::Relaxed),
            Self::DATA_SECONDARY => self.secondary_effect.store(value, Ordering::Relaxed),
            _ => {}
        }
        self.mark_dirty();
    }

    #[must_use]
    pub const fn is_valid_primary_effect(effect_id: i32, levels: i32) -> bool {
        match effect_id {
            // Speed (1), Haste (3)
            1 | 3 => levels >= 1,
            // Resistance (11), Jump Boost (8)
            11 | 8 => levels >= 2,
            // Strength (5)
            5 => levels >= 3,
            _ => false,
        }
    }

    #[must_use]
    pub const fn is_valid_secondary_effect(
        primary_id: i32,
        secondary_id: i32,
        levels: i32,
    ) -> bool {
        if secondary_id <= 0 {
            return true;
        }
        if levels < 4 {
            return false;
        }
        // Regeneration (10) or identical to primary
        secondary_id == 10 || secondary_id == primary_id
    }

    #[must_use]
    pub fn validate_effects(primary: Option<i32>, secondary: Option<i32>, levels: i32) -> bool {
        let primary_id = primary.unwrap_or(0);
        let secondary_id = secondary.unwrap_or(0);

        if primary_id > 0 && !Self::is_valid_primary_effect(primary_id, levels) {
            return false;
        }

        if secondary_id > 0 && !Self::is_valid_secondary_effect(primary_id, secondary_id, levels) {
            return false;
        }

        true
    }

    pub fn update_base(&self, world: &Arc<World>) -> i32 {
        let x = self.position.0.x;
        let y = self.position.0.y;
        let z = self.position.0.z;

        let mut current_level = 0;

        for level in 1..=4 {
            let layer_y = y - level;
            if layer_y < world.dimension.min_y {
                break;
            }

            let mut layer_valid = true;
            for dx in -level..=level {
                for dz in -level..=level {
                    let block_pos = BlockPos::new(x + dx, layer_y, z + dz);
                    let state = world.get_block_state(&block_pos);
                    let block = world.get_block(&block_pos);

                    if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEACON_BASE_BLOCKS) {
                        layer_valid = false;
                        break;
                    }

                    // Optional: Stricter block type validations can happen here
                    let _ = state;
                }
                if !layer_valid {
                    break;
                }
            }

            if layer_valid {
                current_level = level;
            } else {
                break;
            }
        }

        current_level
    }

    pub fn apply_effects(&self, world: &Arc<World>, levels: i32) {
        let primary_id = self.primary_effect.load(Ordering::Relaxed);
        let secondary_id = self.secondary_effect.load(Ordering::Relaxed);

        if primary_id <= 0 {
            return;
        }

        let primary_effect = StatusEffect::from_id(primary_id as u16);
        let secondary_effect = StatusEffect::from_id(secondary_id as u16);

        // Vanilla duration: (9 + levels * 2) * 20 ticks
        let duration_ticks = (9 + levels * 2) * 20;

        // Base amplifier: primary gets amp 1 (Level II) if secondary matches primary
        let base_amp = i32::from(levels >= 4 && primary_id == secondary_id);

        // Vanilla Range is level * 10 + 10 blocks in each horizontal direction
        let range = f64::from(levels * 10 + 10);
        let pos = self.position.0.to_f64();
        let box_min = [pos.x - range, pos.y - range, pos.z - range];
        let box_max = [
            pos.x + range + 1.0,
            pos.y + range + 1.0 + 384.0,
            pos.z + range + 1.0,
        ];
        let bounds = BoundingBox::new_array(box_min, box_max);

        // Apply effect to all players in range
        let players = world.players.load();
        for player in players.iter() {
            if !bounds.intersects(&player.living_entity.entity.bounding_box.load()) {
                continue;
            }

            if let Some(effect) = primary_effect {
                player.add_effect(pumpkin_data::potion::Effect {
                    effect_type: effect,
                    duration: duration_ticks,
                    amplifier: base_amp as u8,
                    ambient: true,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }

            if levels >= 4
                && primary_id != secondary_id
                && let Some(effect) = secondary_effect
            {
                player.add_effect(pumpkin_data::potion::Effect {
                    effect_type: effect,
                    duration: duration_ticks,
                    amplifier: 0,
                    ambient: true,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
    }
}

impl BlockEntity for BeaconBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        // Aligning to strict vanilla NBT tags
        let primary = nbt.get_int("primary_effect").unwrap_or(-1);
        let secondary = nbt.get_int("secondary_effect").unwrap_or(-1);
        let levels = nbt.get_int("Levels").unwrap_or(0); // Vanilla uses capital L
        let custom_name = nbt
            .get_string("CustomName")
            .map(std::string::ToString::to_string);
        let lock_key = nbt.get_string("Lock").map(std::string::ToString::to_string);

        Self {
            position,
            primary_effect: AtomicI32::new(primary),
            secondary_effect: AtomicI32::new(secondary),
            levels: AtomicI32::new(levels),
            dirty: AtomicBool::new(false),
            payment: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            custom_name: Mutex::new(custom_name),
            lock_key: Mutex::new(lock_key),
            last_check_y: AtomicI32::new(position.0.y - 1),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            "primary_effect",
            self.primary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "secondary_effect",
            self.secondary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int("Levels", self.levels.load(Ordering::Relaxed));

        if let Some(name) = &*self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Some(lock) = &*self
            .lock_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            nbt.put_string("Lock", lock.clone());
        }
    }

    fn tick(&self, world: &Arc<World>) {
        // Check properties every 80 ticks matching Java
        if world.get_time_of_day() % 80 == 0 {
            let levels = self.update_base(world);
            self.levels.store(levels, Ordering::Relaxed);

            if levels > 0 {
                self.apply_effects(world, levels);
            }
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int(
            "primary_effect",
            self.primary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int(
            "secondary_effect",
            self.secondary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int("Levels", self.levels.load(Ordering::Relaxed));
        if let Ok(name) = self.custom_name.try_lock()
            && let Some(ref name) = *name
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Ok(lock) = self.lock_key.try_lock()
            && let Some(ref lock) = *lock
        {
            nbt.put_string("Lock", lock.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Inventory for BeaconBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.payment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot == 0 {
            self.payment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot == 0 {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self
                .payment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::swap(&mut removed, &mut *guard);
            self.mark_dirty();
            removed
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot == 0 {
            let mut stack = self
                .payment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.is_empty() {
                return ItemStack::EMPTY.clone();
            }
            let res = stack.split(amount);
            self.mark_dirty();
            res
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot == 0 {
            *self
                .payment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
            self.mark_dirty();
        }
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for BeaconBlockEntity {
    fn clear(&self) {
        if let Ok(mut payment) = self.payment.try_lock() {
            *payment = ItemStack::EMPTY.clone();
        }
    }
}
