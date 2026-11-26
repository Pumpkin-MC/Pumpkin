use std::any::Any;
use std::sync::{
    Arc, Mutex as StdMutex, RwLock,
    atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
};

use crate::block::entities::PropertyDelegate;
use pumpkin_data::data_component_impl::BrewingFuelImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::potion_brewing::BREWING_RECIPES;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_inventory::{Inventory, sync_read_items_from_nbt, sync_write_items_to_nbt};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct BrewingStandBlockEntity {
    pub position: BlockPos,
    world: std::sync::OnceLock<std::sync::Weak<crate::world::World>>,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
    pub brew_time: AtomicI32,
    pub total_brew_time: AtomicI32,
    pub fuel: AtomicI32,
    pub total_fuel: AtomicI32,
    pub speed_multiplier_bits: AtomicU32,
    pub last_potion_count: StdMutex<Option<[bool; 3]>>,
    pub ingredient_item: StdMutex<Option<&'static pumpkin_data::item::Item>>,
    pub metadata: super::ContainerMetadata,
}

impl BrewingStandBlockEntity {
    pub const INVENTORY_SIZE: usize = 5; // 3 potion slots + 1 ingredient + 1 fuel
    pub const ID: &'static str = "minecraft:brewing_stand";
    const DEFAULT_BREW_TICKS: f32 = 400.0;
    const DEFAULT_FUEL_USES: i32 = 20;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        use std::array::from_fn;
        Self {
            position,
            world: std::sync::OnceLock::new(),
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            brew_time: AtomicI32::new(0),
            total_brew_time: AtomicI32::new(0),
            fuel: AtomicI32::new(0),
            total_fuel: AtomicI32::new(0),
            speed_multiplier_bits: AtomicU32::new(1.0f32.to_bits()),
            last_potion_count: StdMutex::new(None),
            ingredient_item: StdMutex::new(None),
            metadata: super::ContainerMetadata::default(),
        }
    }

    fn effective_brew_ticks(&self) -> i32 {
        let speed = f32::from_bits(self.speed_multiplier_bits.load(Ordering::Relaxed));
        let clamped_speed = if speed > 0.0 { speed } else { 1.0 };
        (Self::DEFAULT_BREW_TICKS / clamped_speed).ceil() as i32
    }

    fn is_dynamic_reagent(&self, stack: &ItemStack) -> bool {
        let Some(world) = self.world.get().and_then(std::sync::Weak::upgrade) else {
            return false;
        };
        let Some(server) = world.server.upgrade() else {
            return false;
        };
        let key = format!("minecraft:{}", stack.get_item().registry_key);
        server
            .recipe_manager
            .get_dynamic_recipes_internal()
            .iter()
            .any(|recipe| matches!(recipe, DynamicRecipe::Brewing(r) if r.reagent == key))
    }

    fn is_dynamic_potion_input(&self, stack: &ItemStack) -> bool {
        let Some(world) = self.world.get().and_then(std::sync::Weak::upgrade) else {
            return false;
        };
        let Some(server) = world.server.upgrade() else {
            return false;
        };
        let key = format!("minecraft:{}", stack.get_item().registry_key);
        server
            .recipe_manager
            .get_dynamic_recipes_internal()
            .iter()
            .any(|recipe| matches!(recipe, DynamicRecipe::Brewing(r) if r.input_item == key))
    }

    /// Vanilla `setChanged` only persists the entity, comparators react to slot changes instead.
    fn mark_timer_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Check if the current ingredient matches the stored ingredient
    fn ingredient_matches(&self, ingredient: &ItemStack) -> bool {
        self.ingredient_item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some_and(|stored| !ingredient.is_empty() && ingredient.get_item().id == stored.id)
    }

    /// Check if any potion slot has a valid recipe with the ingredient
    fn is_brewable(&self, ingredient: &ItemStack, world: &Arc<crate::world::World>) -> bool {
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

            let slot_item_id = slot.get_item().id;
            let potion_id = slot
                .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                .and_then(|pc| pc.potion_id);

            // 1. Check data-driven BREWING_RECIPES from datapack
            for recipe in &BREWING_RECIPES {
                if slot_item_id == recipe.from_item().id
                    && ingredient_id == recipe.ingredient().id
                    && potion_id == Some(recipe.from_potion().id as i32)
                {
                    return true;
                }
            }

            // 2. Check dynamic brewing recipes from loaded datapacks
            if let Some(server) = world.server.upgrade() {
                let dynamic_recipes = server.recipe_manager.get_dynamic_recipes_internal();
                let slot_key = format!("minecraft:{}", slot.get_item().registry_key);
                let ing_key = format!("minecraft:{}", ingredient.get_item().registry_key);
                for dyn_recipe in &dynamic_recipes {
                    if let DynamicRecipe::Brewing(r) = dyn_recipe
                        && r.input_item == slot_key
                        && r.reagent == ing_key
                    {
                        if let Some(req_pot) = &r.input_potion {
                            let current_pot = potion_id
                                .and_then(|id| Potion::from_id(id as u8))
                                .map(|p| format!("minecraft:{}", p.name));
                            if current_pot.as_deref() != Some(req_pot.as_str()) {
                                continue;
                            }
                        }
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

        // Fire BrewEvent before mutating items
        if let Some(server) = world.server.upgrade() {
            let mut brew_event = crate::plugin::api::events::inventory::brew::BrewEvent::new(
                self.position,
                self.fuel.load(Ordering::Relaxed) as u8,
            );
            brew_event = server.plugin_manager.fire_blocking(&server, brew_event);
            if brew_event.cancelled {
                return;
            }
        }

        // Brew potion slots
        let mut ingredient_used = false;
        let mut dropped_remainder = None;
        {
            let Ok(mut items) = self.items.write() else {
                return;
            };

            for slot_idx in 0..3usize {
                let slot = &mut items[slot_idx];
                if slot.is_empty() {
                    continue;
                }

                let slot_item_id = slot.get_item().id;
                let potion_id = slot
                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                    .and_then(|pc| pc.potion_id);

                // 1. Try data-driven BREWING_RECIPES
                let mut item_brewed = false;
                for recipe in &BREWING_RECIPES {
                    if slot_item_id == recipe.from_item().id
                        && recipe.ingredient().id == ingredient_id
                        && potion_id == Some(recipe.from_potion().id as i32)
                    {
                        let mut new_slot = ItemStack::new(1, recipe.to_item());
                        let mut pc = slot
                            .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                            .cloned()
                            .unwrap_or_else(|| pumpkin_data::data_component_impl::PotionContentsImpl {
                                potion_id: None,
                                custom_color: None,
                                custom_effects: Vec::new(),
                                custom_name: None,
                            });
                        pc.potion_id = Some(recipe.to_potion().id as i32);
                        new_slot.set_data_component(pc);
                        *slot = new_slot;
                        item_brewed = true;
                        ingredient_used = true;
                        break;
                    }
                }

                if item_brewed {
                    continue;
                }

                // 2. Try dynamic brewing recipes from loaded datapacks
                if let Some(server) = world.server.upgrade() {
                    let dynamic_recipes = server.recipe_manager.get_dynamic_recipes_internal();
                    let slot_key = format!("minecraft:{}", slot.get_item().registry_key);
                    let ing_key = format!("minecraft:{}", ingredient.get_item().registry_key);
                    for dyn_recipe in &dynamic_recipes {
                        if let DynamicRecipe::Brewing(r) = dyn_recipe
                            && r.input_item == slot_key
                            && r.reagent == ing_key
                        {
                            if let Some(req_pot) = &r.input_potion {
                                let current_pot = potion_id
                                    .and_then(|id| Potion::from_id(id as u8))
                                    .map(|p| format!("minecraft:{}", p.name));
                                if current_pot.as_deref() != Some(req_pot.as_str()) {
                                    continue;
                                }
                            }
                            if let Some(target_item) = Item::from_registry_key(
                                r.output_item
                                    .strip_prefix("minecraft:")
                                    .unwrap_or(&r.output_item),
                            ) {
                                let mut new_slot = ItemStack::new(1, target_item);
                                let mut pc = slot
                                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                                    .cloned()
                                    .unwrap_or_else(|| pumpkin_data::data_component_impl::PotionContentsImpl {
                                        potion_id: None,
                                        custom_color: None,
                                        custom_effects: Vec::new(),
                                        custom_name: None,
                                    });
                                if let Some(out_pot_name) = &r.output_potion
                                    && let Some(out_pot) = Potion::from_name(
                                        out_pot_name
                                            .strip_prefix("minecraft:")
                                            .unwrap_or(out_pot_name),
                                    )
                                {
                                    pc.potion_id = Some(out_pot.id as i32);
                                }
                                new_slot.set_data_component(pc);
                                *slot = new_slot;
                                ingredient_used = true;
                                break;
                            }
                        }
                    }
                }
            }

            // Consume one ingredient
            if ingredient_used {
                let remainder = items[3].crafting_remainder();
                items[3].decrement(1);
                if let Some(remainder) = remainder {
                    if items[3].is_empty() {
                        items[3] = remainder;
                    } else {
                        dropped_remainder = Some(remainder);
                    }
                }
            }
        }

        if let Some(remainder) = dropped_remainder {
            world.drop_stack(&self.position, remainder);
        }

        if !ingredient_used {
            return;
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
            if self.fuel.load(Ordering::Relaxed) > 0 && self.is_brewable(&ingredient, world) {
                let next_ticks = self.effective_brew_ticks();
                self.fuel.fetch_sub(1, Ordering::Relaxed);
                self.brew_time.store(next_ticks, Ordering::Relaxed);
                self.total_brew_time.store(next_ticks, Ordering::Relaxed);
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

    fn try_refill_fuel(&self, world: &Arc<crate::world::World>) -> bool {
        let (expected_fuel, default_uses, speed_multiplier) = if self.fuel.load(Ordering::Relaxed)
            <= 0
            && let Ok(items) = self.items.try_read()
            && !items[4].is_empty()
            && let Some(fuel_comp) = items[4].get_data_component::<BrewingFuelImpl>()
        {
            (
                items[4].clone(),
                fuel_comp.get_uses(),
                fuel_comp.get_speed_multiplier(),
            )
        } else {
            return false;
        };

        let initial_power = u16::try_from(default_uses.max(0)).unwrap_or(u16::MAX);
        let fuel_power = if let Some(server) = world.server.upgrade() {
            let mut fuel_event = crate::plugin::api::events::inventory::brewing_stand_fuel::BrewingStandFuelEvent::new(
                self.position,
                initial_power,
            );
            fuel_event = server.plugin_manager.fire_blocking(&server, fuel_event);

            if fuel_event.cancelled {
                return false;
            }

            if fuel_event.fuel_power == initial_power {
                default_uses
            } else {
                i32::from(fuel_event.fuel_power)
            }
        } else {
            default_uses
        };

        let mut dropped_remainder = None;
        let refilled = if self.fuel.load(Ordering::Relaxed) <= 0
            && let Ok(mut items) = self.items.try_write()
            && !items[4].is_empty()
            && items[4].are_equal(&expected_fuel)
        {
            let remainder = items[4].crafting_remainder();
            self.fuel.store(fuel_power, Ordering::Relaxed);
            self.total_fuel.store(fuel_power, Ordering::Relaxed);
            self.speed_multiplier_bits
                .store(speed_multiplier.to_bits(), Ordering::Relaxed);
            items[4].decrement(1);
            if let Some(remainder) = remainder {
                if items[4].is_empty() {
                    items[4] = remainder;
                } else {
                    dropped_remainder = Some(remainder);
                }
            }
            // The fuel slot shrank, comparators needs an update.
            self.comparator_dirty.store(true, Ordering::Relaxed);
            true
        } else {
            false
        };

        if let Some(remainder) = dropped_remainder {
            world.drop_stack(&self.position, remainder);
        }

        refilled
    }
}

impl pumpkin_inventory::Inventory for BrewingStandBlockEntity {
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

    fn update_stack(&self, slot: usize, update: &mut dyn FnMut(&mut ItemStack) -> bool) -> bool {
        pumpkin_inventory::update_locked_stack(&self.items, slot, update)
    }

    fn on_open(&self) {}

    fn on_close(&self) {}

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_valid_slot_for(&self, slot: usize, stack: &ItemStack) -> bool {
        if stack.is_empty() {
            return false;
        }

        match slot {
            // Slots 0-2: valid potion input, only when the bottle slot is currently empty.
            0..=2 => {
                self.get_stack(slot).is_empty()
                    && (pumpkin_inventory::brewing::brewing_screen_handler::is_potion_item(
                        stack.get_item(),
                    ) || BREWING_RECIPES
                        .iter()
                        .any(|r| r.from_item().id == stack.get_item().id)
                        || self.is_dynamic_potion_input(stack))
            }
            // Slot 3: brewing reagent.
            3 => {
                BREWING_RECIPES
                    .iter()
                    .any(|r| r.ingredient().id == stack.get_item().id)
                    || self.is_dynamic_reagent(stack)
            }
            // Slot 4: fuel (`minecraft:brewing_fuel` data component, 26.3+).
            4 => stack.get_data_component::<BrewingFuelImpl>().is_some(),
            _ => false,
        }
    }

    fn is_sided(&self) -> bool {
        true
    }

    fn get_slots_for_face(&self, direction: pumpkin_data::BlockDirection) -> &'static [usize] {
        match direction {
            pumpkin_data::BlockDirection::Up => &[3],
            pumpkin_data::BlockDirection::Down => &[0, 1, 2, 3],
            _ => &[0, 1, 2, 4],
        }
    }

    fn can_take_item_through_face(
        &self,
        slot: usize,
        stack: &ItemStack,
        _direction: pumpkin_data::BlockDirection,
    ) -> bool {
        if slot == 3 {
            stack.get_item().id == Item::GLASS_BOTTLE.id
        } else {
            true
        }
    }
}

impl pumpkin_inventory::Clearable for BrewingStandBlockEntity {
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
        entity.metadata = super::ContainerMetadata::from_nbt(nbt);

        // Load brew time / fuel if present in NBT (26.3 writes Ints; legacy worlds used Short/Byte)
        if let Some(bt) = nbt
            .get_int("BrewTime")
            .or_else(|| nbt.get_short("BrewTime").map(i32::from))
        {
            entity.brew_time.store(bt, Ordering::Relaxed);
        }
        entity.total_brew_time.store(
            nbt.get_int("total_brew_time")
                .unwrap_or(Self::DEFAULT_BREW_TICKS as i32),
            Ordering::Relaxed,
        );
        if let Some(f) = nbt
            .get_int("Fuel")
            .or_else(|| nbt.get_byte("Fuel").map(i32::from))
        {
            entity.fuel.store(f, Ordering::Relaxed);
        }
        entity.total_fuel.store(
            nbt.get_int("total_fuel").unwrap_or(Self::DEFAULT_FUEL_USES),
            Ordering::Relaxed,
        );
        if let Some(speed) = nbt.get_float("speed_multiplier") {
            entity
                .speed_multiplier_bits
                .store(speed.to_bits(), Ordering::Relaxed);
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
        // Persist brew state (`BrewingStandBlockEntity.saveAdditional` in 26.3)
        nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
        nbt.put_int(
            "total_brew_time",
            self.total_brew_time.load(Ordering::Relaxed),
        );
        self.write_inventory_nbt(nbt, true);
        nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));
        nbt.put_int("total_fuel", self.total_fuel.load(Ordering::Relaxed));
        nbt.put_float(
            "speed_multiplier",
            f32::from_bits(self.speed_multiplier_bits.load(Ordering::Relaxed)),
        );
        self.metadata.write_nbt(nbt);
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        self.metadata.apply_item_components(stack);
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::apply_container_items_to_slice(stack, &mut *items);
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_name_only(stack);
    }

    fn collect_pick_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_item_components(stack);
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::collect_container_items_from_slice(&*items, stack);
    }

    fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        self.metadata.get_lock()
    }

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.metadata.get_custom_name()
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
        nbt.put_int(
            "total_brew_time",
            self.total_brew_time.load(Ordering::Relaxed),
        );
        nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));
        nbt.put_int("total_fuel", self.total_fuel.load(Ordering::Relaxed));
        nbt.put_float(
            "speed_multiplier",
            f32::from_bits(self.speed_multiplier_bits.load(Ordering::Relaxed)),
        );
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(&*items, &mut nbt);
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        Some(build_brewing_stand_actor(
            self.position,
            self.brew_time.load(Ordering::Relaxed),
            self.fuel.load(Ordering::Relaxed),
            None,
        ))
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

    fn set_world(&self, world: std::sync::Weak<crate::world::World>) {
        let _ = self.world.set(world);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, world: &Arc<crate::world::World>) {
        let _ = self.world.set(Arc::downgrade(world));
        // Refill fuel counter from fuel item if needed
        let fuel_refilled = self.try_refill_fuel(world);

        // Get current ingredient and check brewing state
        let Ok(items) = self.items.try_read() else {
            return;
        };
        let ingredient = items[3].clone();
        drop(items);
        let brewable = self.is_brewable(&ingredient, world);
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
                self.mark_timer_dirty();
            } else {
                // Continue brewing
                self.mark_timer_dirty();
            }
        } else if brewable && self.fuel.load(Ordering::Relaxed) > 0 {
            let default_ticks = self.effective_brew_ticks();
            let brew_time = if let Some(server) = world.server.upgrade() {
                let mut start_event =
                    crate::plugin::api::events::block::brewing_start::BrewingStartEvent::new(
                        self.position,
                        world.clone(),
                        default_ticks,
                    );
                start_event = server.plugin_manager.fire_blocking(&server, start_event);
                if start_event.cancelled {
                    return;
                }
                start_event.brewing_time
            } else {
                default_ticks
            };

            // Start new brewing cycle
            self.fuel.fetch_sub(1, Ordering::Relaxed);
            self.brew_time.store(brew_time, Ordering::Relaxed);
            self.total_brew_time.store(brew_time, Ordering::Relaxed);
            *self
                .ingredient_item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ingredient.get_item());
            self.mark_timer_dirty();
        } else if fuel_refilled {
            // Mark dirty if fuel was refilled to update fuel indicator
            self.mark_timer_dirty();
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

            // Also mark dirty so inventory/container updates are sent to open screens.
            // The slot change that flipped these bits already flagged the comparator.
            self.mark_timer_dirty();
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
            2 => self.total_brew_time.load(Ordering::Relaxed),
            3 => self.total_fuel.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        match index {
            0 => self.brew_time.store(value, Ordering::Relaxed),
            1 => self.fuel.store(value, Ordering::Relaxed),
            2 => self.total_brew_time.store(value, Ordering::Relaxed),
            3 => self.total_fuel.store(value, Ordering::Relaxed),
            _ => {}
        }
    }

    fn get_properties_size(&self) -> i32 {
        4
    }
}

/// Builds the Bedrock block-actor NBT for brewing stands.
#[must_use]
pub fn build_brewing_stand_actor(
    position: BlockPos,
    cook_time: i32,
    fuel: i32,
    custom_name: Option<&str>,
) -> NbtCompound {
    super::bedrock::BedrockBlockActorBuilder::new("BrewingStand", position)
        .custom_name(custom_name)
        .put_short("CookTime", cook_time as i16)
        .put_short("FuelAmount", fuel as i16)
        .put_short("FuelTotal", 20)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brewing_stand_actor_builder() {
        let pos = BlockPos::new(9, 64, 9);
        let nbt = build_brewing_stand_actor(pos, 200, 15, Some("Brew"));
        assert_eq!(nbt.get_string("id"), Some("BrewingStand"));
        assert_eq!(nbt.get_short("CookTime"), Some(200));
        assert_eq!(nbt.get_short("FuelAmount"), Some(15));
        assert_eq!(nbt.get_short("FuelTotal"), Some(20));
        assert_eq!(nbt.get_string("CustomName"), Some("Brew"));
    }
}
