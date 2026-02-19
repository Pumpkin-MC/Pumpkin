use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::AtomicI32,
    atomic::{AtomicBool, Ordering},
};

use crate::block::entities::PropertyDelegate;
use crate::inventory::Inventory;
use crate::item::ItemStack;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item::Item;
use pumpkin_data::potion_brewing::{ITEM_RECIPES, POTION_RECIPES};
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

pub struct BrewingStandBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
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
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            brew_time: AtomicI32::new(0),
            fuel: AtomicI32::new(0),
            last_potion_count: StdMutex::new(None),
            ingredient_item: StdMutex::new(None),
        }
    }

    /// Check if the current ingredient matches the stored ingredient
    async fn ingredient_matches(&self, ingredient: &ItemStack) -> bool {
        if let Some(stored) = *self.ingredient_item.lock().unwrap() {
            !ingredient.is_empty() && ingredient.get_item().id == stored.id
        } else {
            false
        }
    }

    /// Check if any potion slot has a valid recipe with the ingredient
    async fn is_brewable(&self, ingredient: &ItemStack) -> bool {
        if ingredient.is_empty() {
            return false;
        }

        let ingredient_id = ingredient.get_item().id;

        // Check potion recipes (water bottle -> potions, potion upgrades, etc.)
        for slot_idx in 0..3usize {
            let slot = self.items[slot_idx].lock().await;
            if slot.is_empty() {
                continue;
            }

            // Check item recipes first (potion -> splash potion, splash -> lingering)
            for recipe in ITEM_RECIPES.iter() {
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
                for recipe in POTION_RECIPES.iter() {
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

    /// Perform brewing on all valid potion slots
    async fn do_brew(&self, ingredient: &ItemStack) {
        let ingredient_id = ingredient.get_item().id;

        // Apply recipes to each slot
        for slot_idx in 0..3usize {
            let slot = self.items[slot_idx].lock().await;
            if slot.is_empty() {
                continue;
            }

            let mut new_stack_opt: Option<ItemStack> = None;

            // Try item recipes first (potion -> splash/lingering)
            for recipe in ITEM_RECIPES.iter() {
                if slot.get_item().id == recipe.from().id
                    && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                {
                    let new_item = recipe.to();
                    let potion_comp = slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().cloned();
                    let new_stack = if let Some(pc) = potion_comp {
                        ItemStack::new_with_component(
                            slot.item_count,
                            new_item,
                            vec![(
                                pumpkin_data::data_component::DataComponent::PotionContents,
                                Some(pc.to_dyn()),
                            )],
                        )
                    } else {
                        ItemStack::new(slot.item_count, new_item)
                    };
                    new_stack_opt = Some(new_stack);
                    break;
                }
            }

            // Try potion recipes (modify potion type) if item recipe didn't apply
            if new_stack_opt.is_none()
                && let Some(pc) = slot
                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                && let Some(potion_id) = pc.potion_id
            {
                for recipe in POTION_RECIPES.iter() {
                    if recipe.from().id as i32 == potion_id
                        && recipe.ingredient().iter().any(|i| i.id == ingredient_id)
                    {
                        let new_pc = pumpkin_data::data_component_impl::PotionContentsImpl {
                            potion_id: Some(recipe.to().id as i32),
                            custom_color: pc.custom_color,
                            custom_effects: pc.custom_effects.clone(),
                            custom_name: pc.custom_name.clone(),
                        };
                        let new_stack = ItemStack::new_with_component(
                            slot.item_count,
                            slot.get_item(),
                            vec![(
                                pumpkin_data::data_component::DataComponent::PotionContents,
                                Some(new_pc.to_dyn()),
                            )],
                        );
                        new_stack_opt = Some(new_stack);
                        break;
                    }
                }
            }

            // Drop the lock before calling set_stack
            drop(slot);

            // Update the slot using set_stack if a recipe was applied
            if let Some(new_stack) = new_stack_opt {
                self.set_stack(slot_idx, new_stack).await;
            }
        }

        // Consume ingredient
        let mut ingredient_slot = self.items[3].lock().await;
        ingredient_slot.decrement(1);
        let updated_ingredient = ingredient_slot.clone();
        drop(ingredient_slot);

        // Update the slot with the decremented stack
        self.set_stack(3, updated_ingredient).await;

        // Mark dirty to trigger update
        self.mark_dirty();

        // Handle crafting remainder (like glass bottles from honey bottles)
        // TODO: Implement remainder handling when item lookup by ID is available
    }
}

impl crate::inventory::Inventory for BrewingStandBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> crate::inventory::InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.items {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(
        &self,
        slot: usize,
    ) -> crate::inventory::InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> crate::inventory::InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(
        &self,
        slot: usize,
        amount: u8,
    ) -> crate::inventory::InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut guard = self.items[slot].lock().await;
            let mut taken = ItemStack::EMPTY.clone();
            if guard.item_count <= amount {
                std::mem::swap(&mut taken, &mut *guard);
            } else {
                taken = guard.clone();
                taken.item_count = amount;
                guard.item_count -= amount;
            }
            taken
        })
    }

    fn set_stack(
        &self,
        slot: usize,
        stack: ItemStack,
    ) -> crate::inventory::InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
            self.mark_dirty();
        })
    }

    fn on_open(&self) -> crate::inventory::InventoryFuture<'_, ()> {
        Box::pin(async move {})
    }

    fn on_close(&self) -> crate::inventory::InventoryFuture<'_, ()> {
        Box::pin(async move {})
    }

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
                if let Some(is_brewing_fuel) =
                    stack.get_item().is_tagged_with("minecraft:brewing_fuel")
                    && is_brewing_fuel
                {
                    return false; // Fuel should not go in ingredient slot
                }
                // Allow any item that's not fuel (ingredient validation happens during brewing)
                true
            }
            // Slot 4 - fuel
            4 => stack
                .get_item()
                .is_tagged_with("minecraft:brewing_fuel")
                .is_some_and(|b| b),
            _ => false,
        }
    }
}

impl crate::inventory::Clearable for BrewingStandBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                let mut guard = slot.lock().await;
                guard.clear();
            }
            self.mark_dirty();
        })
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
        let entity = Self::new(position);

        // Load brew time / fuel if present in NBT
        if let Some(bt) = nbt.get_int("BrewTime") {
            entity.brew_time.store(bt, Ordering::Relaxed);
        }
        if let Some(f) = nbt.get_int("Fuel") {
            entity.fuel.store(f, Ordering::Relaxed);
        }

        // Load inventory items from NBT
        entity.read_data(nbt, &entity.items);

        // If there's an ingredient in slot 3, remember its base item for matching
        if let Ok(guard) = entity.items[3].try_lock()
            && !guard.is_empty()
        {
            *entity.ingredient_item.lock().unwrap() = Some(guard.get_item());
        }

        // Recompute last_potion_count so visuals are correct after load
        let mut current: [bool; 3] = [false; 3];
        for (i, slot_arc) in entity.items.iter().take(3).enumerate() {
            if let Ok(slot) = slot_arc.try_lock() {
                current[i] = !slot.is_empty()
                    && (slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().is_some()
                        || slot.get_item().id == pumpkin_data::item::Item::GLASS_BOTTLE.id);
            }
        }
        *entity.last_potion_count.lock().unwrap() = Some(current);

        entity
    }

    fn write_nbt<'a>(
        &'a self,
        _nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Persist brew state
            _nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
            _nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));

            // Save inventory contents to NBT
            self.write_inventory_nbt(_nbt, true).await;
        })
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        // Provide brew state for client-side UI when chunk is sent
        nbt.put_int("BrewTime", self.brew_time.load(Ordering::Relaxed));
        nbt.put_int("Fuel", self.fuel.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tick<'a>(
        &'a self,
        _world: &'a Arc<dyn crate::world::SimpleWorld>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Refill fuel counter from fuel item if needed
            let fuel_refilled = if self.fuel.load(Ordering::Relaxed) <= 0 {
                let fuel_stack_arc = self.items[4].clone();
                let mut fuel_stack = fuel_stack_arc.lock().await;
                if !fuel_stack.is_empty()
                    && fuel_stack
                        .get_item()
                        .is_tagged_with("minecraft:brewing_fuel")
                        .is_some_and(|b| b)
                {
                    self.fuel.store(20, Ordering::Relaxed);
                    fuel_stack.decrement(1);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Get current ingredient and check brewing state
            let ingredient = self.items[3].lock().await.clone();
            let brewable = self.is_brewable(&ingredient).await;
            let is_brewing = self.brew_time.load(Ordering::Relaxed) > 0;

            // Handle brewing state machine
            if is_brewing {
                // Decrement brew time
                let new_brew_time = self.brew_time.fetch_sub(1, Ordering::Relaxed) - 1;
                let is_done_brewing = new_brew_time == 0;

                if is_done_brewing && brewable {
                    // Brewing complete
                    self.do_brew(&ingredient).await;
                } else if !brewable || !self.ingredient_matches(&ingredient).await {
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
                *self.ingredient_item.lock().unwrap() = Some(ingredient.get_item());
                self.mark_dirty();
            } else if fuel_refilled {
                // Mark dirty if fuel was refilled to update fuel indicator
                self.mark_dirty();
            }

            // Ensure clients are notified when potion slot contents (and their data) change.
            // Compute current presence bits for the three bottle slots
            let mut current: [bool; 3] = [false; 3];
            for (i, slot_arc) in self.items.iter().take(3).enumerate() {
                let slot = slot_arc.lock().await;
                // Consider a potion slot "present" when it has an item and a PotionContents component or is a glass bottle
                current[i] = !slot.is_empty() && (slot.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().is_some() || slot.get_item().id == Item::GLASS_BOTTLE.id);
            }

            // If potion presence changed, update last_potion_count and update block state so clients
            let mut needs_update = false;
            {
                let mut last_guard = self.last_potion_count.lock().unwrap();
                if last_guard.is_none() || last_guard.as_ref().unwrap() != &current {
                    *last_guard = Some(current);
                    needs_update = true;
                }
            }

            if needs_update {
                // Update the block state properties for the brewing stand to reflect bottle presence
                let world = _world.clone();
                let (block, state) = world.get_block_and_state(&self.position).await;
                // Use generated block properties helper to produce a new state id with the bits set
                let mut props =
                    pumpkin_data::block_properties::BrewingStandLikeProperties::from_state_id(
                        state.id, block,
                    );
                // Generated field names use raw identifiers for clarity
                props.r#has_bottle_0 = current[0];
                props.r#has_bottle_1 = current[1];
                props.r#has_bottle_2 = current[2];

                world
                    .set_block_state(
                        &self.position,
                        props.to_state_id(block),
                        crate::world::BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                // Also mark dirty so inventory/container updates are sent to open screens
                self.mark_dirty();
            }
        })
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
