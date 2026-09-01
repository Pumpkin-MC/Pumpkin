use std::any::Any;
use std::sync::{
    Arc, Mutex as StdMutex, RwLock,
    atomic::AtomicI32,
    atomic::{AtomicBool, Ordering},
};

use crate::block::entities::PropertyDelegate;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion_brewing::{ITEM_RECIPES, POTION_RECIPES};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::{Inventory, sync_read_items_from_nbt, sync_write_items_to_nbt};

pub struct BrewingStandBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub brew_time: AtomicI32,
    pub fuel: AtomicI32,
    pub last_potion_count: StdMutex<Option<[bool; 3]>>,
    pub ingredient_item: StdMutex<Option<&'static pumpkin_data::item::Item>>,
}

impl BrewingStandBlockEntity {
    pub const INVENTORY_SIZE: usize = 5; // 3 potion slots + 1 ingredient + 1 fuel
    pub const ID: &'static str = "minecraft:brewing_stand";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        use std::array::from_fn;
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            brew_time: AtomicI32::new(0),
            fuel: AtomicI32::new(0),
            last_potion_count: StdMutex::new(None),
            ingredient_item: StdMutex::new(None),
        }
    }

    /// Check if the current ingredient matches the stored ingredient
    fn ingredient_matches(&self, ingredient: &ItemStack) -> bool {
        self.ingredient_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some_and(|stored| !ingredient.is_empty() && ingredient.get_item().id == stored.id)
    }

    /// Check if any potion slot has a valid recipe with the ingredient
    fn is_brewable(&self, ingredient: &ItemStack) -> bool {
        if ingredient.is_empty() {
            return false;
        }

        let ingredient_id = ingredient.get_item().id;

        // Check potion recipes (water bottle -> potions, potion upgrades, etc.)
        let Ok(items) = self.items.read() else {
            return false;
        };
        for slot_idx in 0..3usize {
            let slot = &items[slot_idx];
            if slot.is_empty() {
                continue;
            }

            // Check item recipes first (potion -> splash potion, splash -> lingering)
            for recipe in &ITEM_RECIPES {
                if slot.get_item().id == recipe.from().id
                    && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                {
                    return true;
                }
            }

            // Check potion recipes (modify potion type)
            if let Some(pc) =
                slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                && let Some(potion_id) = pc.potion_id
            {
                for recipe in &POTION_RECIPES {
                    if recipe.from().id as i32 == potion_id
                        && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Perform the brewing process on all valid potion slots
    #[expect(clippy::too_many_lines)]
    fn do_brew(&self, world: &Arc<crate::world::World>, ingredient: &ItemStack) {
        if ingredient.is_empty() {
            return;
        }

        let ingredient_id = ingredient.get_item().id;

        // Brew potion slots
        let mut ingredient_used = false;
        {
            let Ok(mut items) = self.items.write() else {
                return;
            };

            for slot_idx in 0..3usize {
                let slot = &mut items[slot_idx];
                if slot.is_empty() {
                    continue;
                }

                // 1. Try item recipes (e.g. gunpowder -> splash potion, dragon breath -> lingering)
                let mut item_brewed = false;
                for recipe in &ITEM_RECIPES {
                    if slot.get_item().id == recipe.from().id
                        && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                    {
                        // Preserve potion contents component when converting potion type
                        let pc = slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().cloned();
                        *slot = ItemStack::new(1, recipe.to());
                        if let Some(pc) = pc {
                            slot.set_data_component(pc);
                        }
                        item_brewed = true;
                        ingredient_used = true;
                        break;
                    }
                }

                if item_brewed {
                    continue;
                }

                // 2. Try potion recipes (e.g. water bottle -> awkward potion, awkward -> strength)
                if let Some(pc) = slot
                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                    && let Some(potion_id) = pc.potion_id
                {
                    for recipe in &POTION_RECIPES {
                        if recipe.from().id as i32 == potion_id
                            && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                        {
                            let new_potion_id = recipe.to().id as i32;
                            let mut new_pc = pc.clone();
                            new_pc.potion_id = Some(new_potion_id);
                            slot.set_data_component(new_pc);
                            ingredient_used = true;
                            break;
                        }
                    }
                }
            }

            // Consume one ingredient
            if ingredient_used {
                items[3].decrement(1);
            }
        }

        if !ingredient_used {
            return;
        }

        // Fire BrewEvent
        if let Some(server) = world.server.upgrade() {
            let mut brew_event = crate::plugin::api::events::inventory::brew::BrewEvent::new(
                self.position,
                self.fuel.load(Ordering::Relaxed) as u8,
            );
            server
                .plugin_manager
                .fire_blocking(&server, &mut brew_event);
        }

        // Check if remaining ingredient matches or clear it
        if let Ok(items) = self.items.read() {
            let remaining = &items[3];
            if remaining.is_empty()
                || !self
                    .ingredient_item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .is_some_and(|stored| remaining.get_item().id == stored.id)
            {
                *self
                    .ingredient_item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            }
        }

        // Check if we can immediately start the next brew
        if let Ok(items) = self.items.read() {
            let ingredient = items[3].clone();
            drop(items);
            if self.fuel.load(Ordering::Relaxed) > 0 && self.is_brewable(&ingredient) {
                self.fuel.fetch_sub(1, Ordering::Relaxed);
                self.brew_time.store(400, Ordering::Relaxed);
                *self
                    .ingredient_item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some(ingredient.get_item());
            } else {
                self.brew_time.store(0, Ordering::Relaxed);
            }
        }

        // Play sound at the center of the block
        let pos = Vector3::new(
            self.position.0.x as f64 + 0.5,
            self.position.0.y as f64 + 0.5,
            self.position.0.z as f64 + 0.5,
        );
        world.play_sound(Sound::BlockBrewingStandBrew, SoundCategory::Blocks, &pos);

        // Mark dirty to trigger update
        self.mark_dirty();
    }
}

impl pumpkin_world::inventory::Inventory for BrewingStandBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in items.iter() {
            if !slot.is_empty() {
                return false;
            }
        }
        true
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
        let taken = if items[slot].item_count <= amount {
            std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
        } else {
            let mut taken = items[slot].clone();
            taken.item_count = amount;
            items[slot].item_count -= amount;
            taken
        };
        self.mark_dirty();
        taken
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot] = stack;
        self.mark_dirty();
    }

    fn on_open(&self) {}

    fn on_close(&self) {}

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_valid_slot_for(&self, slot: usize, stack: &ItemStack) -> bool {
        if stack.is_empty() {
            return true;
        }

        match slot {
            // Slots 0-2 - potion bottles
            0..=2 => stack
                .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                .is_some(),
            // Slot 3 - ingredient (must be tagged as brewable)
            3 => {
                // Check if item is a valid brewing ingredient
                if stack.get_item().has_tag(&tag::Item::MINECRAFT_BREWING_FUEL) {
                    return false; // Fuel should not go in ingredient slot
                }
                // Allow any item that's not fuel (ingredient validation happens during brewing)
                true
            }
            // Slot 4 - fuel
            4 => stack.get_item().has_tag(&tag::Item::MINECRAFT_BREWING_FUEL),
            _ => false,
        }
    }
}

impl pumpkin_world::inventory::Clearable for BrewingStandBlockEntity {
    fn clear(&self) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}

impl crate::block::entities::BlockEntity for BrewingStandBlockEntity {
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
        let mut entity = Self::new(position);

        // Load brew time / fuel if present in NBT
        if let Some(bt) = nbt.get_int("BrewTime") {
            entity.brew_time.store(bt, Ordering::Relaxed);
        }
        if let Some(f) = nbt.get_int("Fuel") {
            entity.fuel.store(f, Ordering::Relaxed);
        }

        // Load inventory items from NBT
        let items = entity
            .items
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        sync_read_items_from_nbt(nbt, items);

        // If there's an ingredient in slot 3, remember its base item for matching
        let ingredient_item = (!items[3].is_empty()).then(|| items[3].get_item());

        // Recompute last_potion_count so visuals are correct after load
        let mut current: [bool; 3] = [false; 3];
        for (i, slot) in items.iter().take(3).enumerate() {
            current[i] = !slot.is_empty()
                && (slot
                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                    .is_some()
                    || slot.get_item().id == pumpkin_data::item::Item::GLASS_BOTTLE.id);
        }

        if let Some(item) = ingredient_item {
            *entity
                .ingredient_item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(item);
        }

        *entity
            .last_potion_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(current);

        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        // Persist brew state
        nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
        nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));

        // Save inventory contents to NBT
        self.write_inventory_nbt(nbt, true);
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
        nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(&*items, &mut nbt);
        }
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tick(&self, world: &Arc<crate::world::World>) {
        // Refill fuel counter from fuel item if needed
        let fuel_refilled = self.fuel.load(Ordering::Relaxed) <= 0
            && if let Ok(mut items) = self.items.try_write()
                && !items[4].is_empty()
                && items[4]
                    .get_item()
                    .has_tag(&tag::Item::MINECRAFT_BREWING_FUEL)
            {
                if let Some(server) = world.server.upgrade() {
                    let mut fuel_event = crate::plugin::api::events::inventory::brewing_stand_fuel::BrewingStandFuelEvent::new(
                            self.position,
                            20,
                        );
                    server
                        .plugin_manager
                        .fire_blocking(&server, &mut fuel_event);
                }
                self.fuel.store(20, Ordering::Relaxed);
                items[4].decrement(1);
                true
            } else {
                false
            };

        // Get current ingredient and check brewing state
        let Ok(items) = self.items.try_read() else {
            return;
        };
        let ingredient = items[3].clone();
        drop(items);
        let brewable = self.is_brewable(&ingredient);
        let is_brewing = self.brew_time.load(Ordering::Relaxed) > 0;

        // Handle brewing state machine
        if is_brewing {
            // Decrement brew time
            let new_brew_time = self.brew_time.fetch_sub(1, Ordering::Relaxed) - 1;
            let is_done_brewing = new_brew_time == 0;

            if is_done_brewing && brewable {
                // Brewing complete
                self.do_brew(world, &ingredient);
            } else if !brewable || !self.ingredient_matches(&ingredient) {
                // Cancel brewing
                self.brew_time.store(0, Ordering::Relaxed);
                self.mark_dirty();
            } else {
                // Continue brewing
                self.mark_dirty();
            }
        } else if brewable && self.fuel.load(Ordering::Relaxed) > 0 {
            // Start new brewing cycle
            self.fuel.fetch_sub(1, Ordering::Relaxed);
            self.brew_time.store(400, Ordering::Relaxed);
            *self
                .ingredient_item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ingredient.get_item());
            self.mark_dirty();
        } else if fuel_refilled {
            // Mark dirty if fuel was refilled to update fuel indicator
            self.mark_dirty();
        }

        // Ensure clients are notified when potion slot contents (and their data) change.
        // Compute current presence bits for the three bottle slots
        let mut current: [bool; 3] = [false; 3];
        if let Ok(items_guard) = self.items.try_read() {
            for (i, slot) in items_guard.iter().take(3).enumerate() {
                // Consider a potion slot "present" when it has an item and a PotionContents component or is a glass bottle
                current[i] = !slot.is_empty() && (slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().is_some() || slot.get_item().id == Item::GLASS_BOTTLE.id);
            }
        }

        // If potion presence changed, update last_potion_count and update block state so clients
        let mut needs_update = false;
        {
            let mut last_guard = self
                .last_potion_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if last_guard.as_ref() != Some(&current) {
                *last_guard = Some(current);
                needs_update = true;
            }
        }

        if needs_update {
            // Update the block state properties for the brewing stand to reflect bottle presence
            let (block, state) = world.get_block_and_state(&self.position);
            // Use generated block properties helper to produce a new state id with the bits set
            let mut props =
                pumpkin_data::block_properties::BrewingStandLikeProperties::from_state_id(state.id);
            // Generated field names use raw identifiers for clarity
            props.r#has_bottle_0 = current[0];
            props.r#has_bottle_1 = current[1];
            props.r#has_bottle_2 = current[2];

            world.set_block_state(
                &self.position,
                props.to_state_id(block),
                crate::world::BlockFlags::NOTIFY_ALL,
            );

            // Also mark dirty so inventory/container updates are sent to open screens
            self.mark_dirty();
        }
    }

    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        Some(self as Arc<dyn PropertyDelegate>)
    }
}

impl PropertyDelegate for BrewingStandBlockEntity {
    fn get_property(&self, index: i32) -> i32 {
        match index {
            0 => self.brew_time.load(Ordering::Relaxed),
            1 => self.fuel.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    fn set_property(&self, _index: i32, _value: i32) {}

    fn get_properties_size(&self) -> i32 {
        2
    }
}
