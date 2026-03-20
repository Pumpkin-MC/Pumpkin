use std::sync::Arc;

use pumpkin_data::recipes::CookingRecipe;
use tokio::sync::Mutex;

use crate::{
    block::entities::{BlockEntity, PropertyDelegate},
    inventory::{Clearable, Inventory},
    item::ItemStack,
};

/// Trait for extracting smelting experience from cooking block entities.
/// This is a separate dyn-compatible trait since `CookingBlockEntityBase` is not.
pub trait ExperienceContainer: Send + Sync {
    /// Extract and reset accumulated experience, returning the total as an integer
    fn extract_experience(&self) -> i32;
}

pub trait CookingBlockEntityBase:
    Sync + Send + Inventory + PropertyDelegate + BlockEntity + Clearable
{
    fn get_cooking_time_spent(&self) -> u16;
    fn get_cooking_total_time(&self) -> u16;
    fn get_lit_time_remaining(&self) -> u16;
    fn get_lit_total_time(&self) -> u16;

    /// Track that a recipe was used (for XP calculation on extraction)
    /// Uses the result item ID as the recipe identifier
    fn add_recipe_used(&self, recipe: &CookingRecipe);
    /// Extract and reset accumulated experience, returning the total as an integer
    /// Calculates XP from tracked recipes and clears the `recipes_used` map
    fn extract_experience_from_recipes(&self) -> i32;

    fn get_input_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>>;
    fn get_fuel_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>>;
    fn get_output_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>>;

    fn set_cooking_time_spent(&self, spent_time: u16);
    fn set_cooking_total_time(&self, total_time: u16);
    fn set_lit_time_remaining(&self, remaining_time: u16);
    fn set_lit_total_time(&self, total_time: u16);

    fn is_burning(&self) -> bool;
    fn can_accept_recipe_output(
        &self,
        recipe: Option<&CookingRecipe>,
        max_count: u8,
    ) -> impl Future<Output = bool>;
    fn craft_recipe(&self, recipe: Option<&CookingRecipe>) -> impl Future<Output = bool>;
}

#[macro_export]
macro_rules! impl_cooking_block_entity_base {
    ($struct_name:ty) => {
        impl CookingBlockEntityBase for $struct_name {
            fn get_cooking_time_spent(&self) -> u16 {
                self.cooking_time_spent.load(Ordering::Relaxed)
            }

            fn get_cooking_total_time(&self) -> u16 {
                self.cooking_total_time.load(Ordering::Relaxed)
            }

            fn get_lit_time_remaining(&self) -> u16 {
                self.lit_time_remaining.load(Ordering::Relaxed)
            }

            fn get_lit_total_time(&self) -> u16 {
                self.lit_total_time.load(Ordering::Relaxed)
            }

            fn get_input_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>> {
                let items = self.items.clone();
                async move { items[0].clone() }
            }

            fn get_fuel_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>> {
                let items = self.items.clone();
                async move { items[1].clone() }
            }

            fn get_output_item(&self) -> impl std::future::Future<Output = Arc<Mutex<ItemStack>>> {
                let items = self.items.clone();
                async move { items[2].clone() }
            }

            fn set_cooking_time_spent(&self, spent_time: u16) {
                self.cooking_time_spent.store(spent_time, Ordering::Relaxed);
            }

            fn set_cooking_total_time(&self, total_time: u16) {
                self.cooking_total_time.store(total_time, Ordering::Relaxed);
            }

            fn set_lit_time_remaining(&self, remaining_time: u16) {
                self.lit_time_remaining
                    .store(remaining_time, Ordering::Relaxed);
            }

            fn set_lit_total_time(&self, total_time: u16) {
                self.lit_total_time.store(total_time, Ordering::Relaxed);
            }

            fn is_burning(&self) -> bool {
                self.get_lit_time_remaining() > 0
            }

            fn add_recipe_used(&self, recipe: &pumpkin_data::recipes::CookingRecipe) {
                // Track recipe usage by recipe ID for XP calculation
                let recipe_id = recipe.recipe_id.to_string();
                let mut recipes = self.recipes_used.lock().unwrap();
                *recipes.entry(recipe_id).or_insert(0) += 1;
            }

            fn extract_experience_from_recipes(&self) -> i32 {
                // Calculate total XP from tracked recipes and clear the map (vanilla behavior)
                let mut recipes = self.recipes_used.lock().unwrap();
                let mut total_xp: f32 = 0.0;
                for (recipe_id, count) in recipes.iter() {
                    // Look up the recipe's XP value
                    if let Some(xp) = pumpkin_data::recipes::get_recipe_experience(recipe_id) {
                        total_xp += xp * (*count as f32);
                    }
                }
                recipes.clear();
                total_xp.floor() as i32
            }

            async fn can_accept_recipe_output(
                &self,
                recipe: Option<&pumpkin_data::recipes::CookingRecipe>,
                max_count: u8,
            ) -> bool {
                let recipe = match recipe {
                    Some(cooking_recipe) => cooking_recipe,
                    None => return false,
                };

                let top_item_stack = self.items[0].lock().await;
                let is_top_items_empty = top_item_stack.is_empty();
                drop(top_item_stack);

                let side_item_stack = self.items[2].lock().await;
                if side_item_stack.is_empty() {
                    return !is_top_items_empty;
                }

                if let Some(recipe_output_item) = pumpkin_data::item::Item::from_registry_key(
                    recipe.result.id.strip_prefix("minecraft:").unwrap(),
                ) && !is_top_items_empty
                    && recipe_output_item.id == side_item_stack.item.id
                    && side_item_stack.item_count < max_count
                    && side_item_stack.item_count < side_item_stack.get_max_stack_size()
                {
                    return true;
                }
                false
            }
            async fn craft_recipe(
                &self,
                recipe: Option<&pumpkin_data::recipes::CookingRecipe>,
            ) -> bool {
                let can_accept_output = self
                    .can_accept_recipe_output(recipe, self.get_max_count_per_stack())
                    .await;
                if let Some(recipe) = recipe {
                    if can_accept_output {
                        let mut side_items = self.items[2].lock().await;
                        let output_item = match pumpkin_data::item::Item::from_registry_key(
                            recipe.result.id.strip_prefix("minecraft:").unwrap(),
                        ) {
                            Some(item) => item,
                            None => return false,
                        };
                        let output_item_stack = ItemStack::new(recipe.result.count, output_item);

                        if side_items.are_equal(ItemStack::EMPTY) {
                            drop(side_items);
                            self.set_stack(2, output_item_stack).await;
                        } else if side_items.are_items_and_components_equal(&output_item_stack) {
                            side_items.increment(1);
                        }

                        // Track recipe usage for XP calculation (vanilla RecipesUsed format)
                        self.add_recipe_used(recipe);
                    }

                    let bottom_items = self.items[1].lock().await;
                    let mut top_items = self.items[0].lock().await;
                    if top_items.item.id == pumpkin_data::item::Item::WET_SPONGE.id
                        && !bottom_items.is_empty()
                        && bottom_items.item.id == pumpkin_data::item::Item::BUCKET.id
                    {
                        drop(bottom_items);
                        self.set_stack(
                            1,
                            ItemStack::new(1, &pumpkin_data::item::Item::WATER_BUCKET),
                        )
                        .await;
                    }

                    top_items.decrement(1);
                    return true;
                }

                false
            }
        }
    };
}

#[macro_export]
macro_rules! impl_property_delegate_for_cooking {
    ($struct_name:ty) => {
        impl $crate::block::entities::PropertyDelegate for $struct_name {
            fn get_property(&self, index: i32) -> i32 {
                match index {
                    0 => self.get_lit_time_remaining() as i32,
                    1 => self.get_lit_total_time() as i32,
                    2 => self.get_cooking_time_spent() as i32,
                    3 => self.get_cooking_total_time() as i32,
                    _ => 0,
                }
            }

            fn set_property(&self, index: i32, value: i32) {
                let value = value as u16;
                match index {
                    0 => self.set_lit_time_remaining(value),
                    1 => self.set_lit_total_time(value),
                    2 => self.set_cooking_time_spent(value),
                    3 => self.set_cooking_total_time(value),
                    _ => {}
                }
            }

            fn get_properties_size(&self) -> i32 {
                4
            }
        }
    };
}

#[macro_export]
macro_rules! impl_clearable_for_cooking {
    ($struct_name:ty) => {
        impl $crate::inventory::Clearable for $struct_name {
            fn clear(&self) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + '_>> {
                Box::pin(async move {
                    for slot in self.items.iter() {
                        *slot.lock().await = ItemStack::EMPTY.clone();
                    }
                    self.mark_dirty();
                })
            }
        }
    };
}

#[macro_export]
macro_rules! impl_experience_container_for_cooking {
    ($struct_name:ty) => {
        impl $crate::block::entities::furnace_like_block_entity::ExperienceContainer
            for $struct_name
        {
            fn extract_experience(&self) -> i32 {
                // Delegate to the CookingBlockEntityBase method
                self.extract_experience_from_recipes()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_inventory_for_cooking {
    ($struct_name:ty) => {
        impl $crate::inventory::Inventory for $struct_name {
            fn size(&self) -> usize {
                self.items.len()
            }

            fn is_empty(&self) -> $crate::inventory::InventoryFuture<'_, bool> {
                Box::pin(async move {
                    for slot in self.items.iter() {
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
            ) -> $crate::inventory::InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
                Box::pin(async move { self.items[slot].clone() })
            }

            fn remove_stack(
                &self,
                slot: usize,
            ) -> $crate::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let mut removed = ItemStack::EMPTY.clone();
                    let mut guard = self.items[slot].lock().await;
                    std::mem::swap(&mut removed, &mut *guard);
                    self.mark_dirty();
                    removed
                })
            }

            fn remove_stack_specific(
                &self,
                slot: usize,
                amount: u8,
            ) -> $crate::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let res = $crate::inventory::split_stack(&self.items, slot, amount).await;
                    self.mark_dirty();
                    res
                })
            }

            fn set_stack(
                &self,
                slot: usize,
                stack: ItemStack,
            ) -> $crate::inventory::InventoryFuture<'_, ()> {
                Box::pin(async move {
                    let furnace_stack = self.get_stack(slot).await;
                    let mut furnace_stack = furnace_stack.lock().await;

                    let is_same_item = !stack.is_empty()
                        && ItemStack::are_items_and_components_equal(&furnace_stack, &stack);

                    *furnace_stack = stack.clone();
                    drop(furnace_stack);

                    if slot == 0 && !is_same_item {
                        if let Some(recipe) =
                            pumpkin_data::recipes::get_cooking_recipe_with_ingredient(
                                stack.item,
                                CookingRecipeKind::Smelting,
                            )
                        {
                            self.set_cooking_total_time(recipe.cookingtime as u16);
                        } else {
                            self.set_cooking_total_time(0);
                        }
                        self.set_cooking_time_spent(0);
                    }

                    // Always consider the inventory changed when setting a stack
                    self.mark_dirty();
                })
            }

            fn mark_dirty(&self) {
                self.dirty.store(true, Ordering::Relaxed);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_block_entity_for_cooking {
    ($struct_name:ty,$recipe_kind:expr) => {
        impl $crate::block::entities::BlockEntity for $struct_name {
            fn tick<'a>(
                &'a self,
                world: &'a Arc<dyn $crate::world::SimpleWorld>,
            ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
                Box::pin(async move {
                    let is_burning = self.is_burning();
                    let mut is_dirty = false;
                    if self.is_burning() {
                        self.lit_time_remaining.fetch_sub(1, Ordering::Relaxed);
                    }

                    let top_items = self.items[0].lock().await;
                    let is_top_items_empty = top_items.is_empty();

                    let furnace_recipe = pumpkin_data::recipes::get_cooking_recipe_with_ingredient(
                        top_items.item,
                        $recipe_kind,
                    );
                    drop(top_items);

                    let can_accept_output = self
                        .can_accept_recipe_output(furnace_recipe, self.get_max_count_per_stack())
                        .await;

                    let bottom_items_is_empty = self.items[1].lock().await.is_empty();
                    if self.is_burning() || !bottom_items_is_empty && !is_top_items_empty {
                        if !self.is_burning() && can_accept_output {
                            let mut bottom_items = self.items[1].lock().await;

                            let base_fuel_ticks =
                                pumpkin_data::fuels::get_item_burn_ticks(bottom_items.item.id)
                                    .unwrap_or(0);

                            let adjusted_fuel_ticks = if matches!(
                                $recipe_kind,
                                CookingRecipeKind::Blasting | CookingRecipeKind::Smoking
                            ) {
                                base_fuel_ticks / 2
                            } else {
                                base_fuel_ticks
                            };

                            self.set_lit_time_remaining(adjusted_fuel_ticks);
                            self.set_lit_total_time(adjusted_fuel_ticks);

                            if self.is_burning() {
                                is_dirty = true;
                                if !bottom_items.is_empty() {
                                    bottom_items.decrement(1);
                                    if let Some(remainder_id) =
                                        pumpkin_data::recipe_remainder::get_recipe_remainder_id(
                                            bottom_items.item.id,
                                        )
                                        && bottom_items.is_empty()
                                        && let Some(remainder_item) =
                                            pumpkin_data::item::Item::from_id(remainder_id)
                                    {
                                        drop(bottom_items);
                                        self.set_stack(1, ItemStack::new(1, remainder_item)).await;
                                    }
                                }
                            }
                        }

                        if self.is_burning() && can_accept_output {
                            self.cooking_time_spent.fetch_add(1, Ordering::Relaxed);

                            if self.get_cooking_time_spent() == self.get_cooking_total_time() {
                                self.set_cooking_time_spent(0);
                                if let Some(cooking_recipe) = furnace_recipe {
                                    let cooking_total_time = cooking_recipe.cookingtime;
                                    self.set_cooking_total_time(cooking_total_time as u16);

                                    self.craft_recipe(Some(cooking_recipe)).await;
                                    is_dirty = true;
                                }
                            }
                        } else {
                            self.set_cooking_time_spent(0);
                        }
                    } else if !self.is_burning() && self.get_cooking_time_spent() > 0 {
                        self.cooking_time_spent
                            .fetch_update(Ordering::Acquire, Ordering::Acquire, |v| {
                                Some(
                                    v.saturating_sub(2)
                                        .min(self.cooking_total_time.load(Ordering::Acquire)),
                                )
                            })
                            .unwrap();
                    }

                    if is_burning != self.is_burning() {
                        is_dirty = true;
                        let world = world.clone();

                        let (furnace_block, furnace_block_state) =
                            world.get_block_and_state(&self.position).await;
                        let mut props =
                            pumpkin_data::block_properties::FurnaceLikeProperties::from_state_id(
                                furnace_block_state.id,
                                furnace_block,
                            );

                        if self.is_burning() {
                            props.lit = true;
                            world
                                .set_block_state(
                                    &self.position,
                                    props.to_state_id(furnace_block),
                                    $crate::world::BlockFlags::NOTIFY_ALL,
                                )
                                .await;
                        } else {
                            props.lit = false;
                            world
                                .set_block_state(
                                    &self.position,
                                    props.to_state_id(furnace_block),
                                    $crate::world::BlockFlags::NOTIFY_ALL,
                                )
                                .await;
                        }
                    }

                    if is_dirty {
                        self.mark_dirty();
                    }
                })
            }

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
                let cooking_total_time = AtomicU16::new(
                    nbt.get_short("cooking_total_time")
                        .map_or(0, |cooking_total_time| cooking_total_time as u16),
                );
                let cooking_time_spent = AtomicU16::new(
                    nbt.get_short("cooking_time_spent")
                        .map_or(0, |cooking_time_spent| cooking_time_spent as u16),
                );
                let lit_total_time = AtomicU16::new(
                    nbt.get_short("lit_total_time")
                        .map_or(0, |lit_total_time| lit_total_time as u16),
                );
                let lit_time_remaining = AtomicU16::new(
                    nbt.get_short("lit_time_remaining")
                        .map_or(0, |lit_time_remaining| lit_time_remaining as u16),
                );
                // Load RecipesUsed from NBT (vanilla format: map of recipe ID -> craft count)
                let mut recipes_used_map = HashMap::new();
                if let Some(recipes_compound) = nbt.get_compound("RecipesUsed") {
                    for (recipe_id, tag) in &recipes_compound.child_tags {
                        if let pumpkin_nbt::tag::NbtTag::Int(count) = tag {
                            recipes_used_map.insert(recipe_id.clone(), *count as u32);
                        }
                    }
                }

                let furnace = Self {
                    position,
                    dirty: AtomicBool::new(false),
                    items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
                    cooking_total_time,
                    cooking_time_spent,
                    lit_total_time,
                    lit_time_remaining,
                    recipes_used: std::sync::Mutex::new(recipes_used_map),
                };
                furnace.read_data(nbt, &furnace.items);

                furnace
            }

            fn write_nbt<'a>(
                &'a self,
                nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
            ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
                Box::pin(async move {
                    nbt.put_short("cooking_total_time", self.get_cooking_total_time() as i16);
                    nbt.put_short("cooking_time_spent", self.get_cooking_time_spent() as i16);
                    nbt.put_short("lit_total_time", self.get_lit_total_time() as i16);
                    nbt.put_short("lit_time_remaining", self.get_lit_time_remaining() as i16);

                    // Save RecipesUsed in vanilla format (map of recipe ID -> craft count)
                    // Scope the mutex guard so it's dropped before the await
                    {
                        let recipes = self.recipes_used.lock().unwrap();
                        if !recipes.is_empty() {
                            let mut recipes_compound = pumpkin_nbt::compound::NbtCompound::new();
                            for (recipe_id, count) in recipes.iter() {
                                recipes_compound.put(
                                    recipe_id.as_str(),
                                    pumpkin_nbt::tag::NbtTag::Int(*count as i32),
                                );
                            }
                            nbt.put(
                                "RecipesUsed",
                                pumpkin_nbt::tag::NbtTag::Compound(recipes_compound),
                            );
                        }
                    }

                    self.write_inventory_nbt(nbt, true).await;
                })
            }

            fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn $crate::inventory::Inventory>> {
                Some(self)
            }

            fn is_dirty(&self) -> bool {
                self.dirty.load(Ordering::Relaxed)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn to_property_delegate(
                self: Arc<Self>,
            ) -> Option<Arc<dyn $crate::block::entities::PropertyDelegate>> {
                Some(self as Arc<dyn $crate::block::entities::PropertyDelegate>)
            }

            fn to_experience_container(
                self: Arc<Self>,
            ) -> Option<
                Arc<dyn $crate::block::entities::furnace_like_block_entity::ExperienceContainer>,
            > {
                Some(
                    self as Arc<
                        dyn $crate::block::entities::furnace_like_block_entity::ExperienceContainer,
                    >,
                )
            }
        }
    };
}
