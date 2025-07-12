use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};

use async_trait::async_trait;
use pumpkin_data::{
    block_properties::{BlockProperties, FurnaceLikeProperties},
    fuels::get_item_burn_ticks,
    item::Item,
    recipes::{CookingRecipe, CookingRecipeType, RECIPES_COOKING},
};
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
    world::{BlockFlags, SimpleWorld},
};

use super::BlockEntity;

#[derive(Debug)]
pub struct FurnaceBlockEntity {
    pub position: BlockPos,
    pub dirty: AtomicBool,

    pub cooking_time_spent: AtomicU16,
    pub cooking_total_time: AtomicU16,
    pub lit_time_remaining: AtomicU16,
    pub lit_total_time: AtomicU16,

    pub items: [Arc<Mutex<ItemStack>>; 3],
}

impl FurnaceBlockEntity {
    #[must_use]
    pub fn is_burning(&self) -> bool {
        self.lit_time_remaining.load(Ordering::Relaxed) > 0
    }

    pub fn get_furnace_cooking_recipe(item: &Item) -> Option<&CookingRecipe> {
        if let Some(recipe_type) = RECIPES_COOKING.iter().find(|recipe| match recipe {
            CookingRecipeType::Smelting(smelting_recipe) => {
                smelting_recipe.ingredient.match_item(item)
            }
            _ => false,
        }) {
            match recipe_type {
                CookingRecipeType::Smelting(cooking_recipe) => {
                    return Some(cooking_recipe);
                }
                _ => {
                    return None;
                }
            }
        }

        None
    }

    async fn can_accept_recipe_output(
        &self,
        recipe: Option<&CookingRecipe>,
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

        if let Some(recipe_output_item) =
            Item::from_registry_key(recipe.result.id.strip_prefix("minecraft:").unwrap())
        {
            if !is_top_items_empty
                && recipe_output_item.id == side_item_stack.item.id
                && side_item_stack.item_count < max_count
                && side_item_stack.item_count < side_item_stack.get_max_stack_size()
            {
                return true;
            }
        }
        false
    }

    async fn craft_recipe(&self, recipe: Option<&CookingRecipe>) -> bool {
        let can_accepet_output = self
            .can_accept_recipe_output(recipe, self.get_max_count_per_stack())
            .await;
        if let Some(recipe) = recipe {
            if can_accepet_output {
                let mut side_items = self.items[2].lock().await;
                let output_item = match Item::from_registry_key(
                    recipe.result.id.strip_prefix("minecraft:").unwrap(),
                ) {
                    Some(item) => item,
                    None => return false,
                };
                let output_item_stack = ItemStack::new(recipe.result.count, output_item);

                if side_items.are_equal(&ItemStack::EMPTY) {
                    drop(side_items);
                    self.set_stack(2, output_item_stack).await;
                } else if side_items.are_items_and_components_equal(&output_item_stack) {
                    side_items.increment(1);
                }
            }

            let bottom_items = self.items[1].lock().await;
            let mut top_items = self.items[0].lock().await;
            if top_items.item.id == Item::WET_SPONGE.id
                && !bottom_items.is_empty()
                && bottom_items.item.id == Item::BUCKET.id
            {
                drop(bottom_items);
                self.set_stack(1, ItemStack::new(1, &Item::WATER_BUCKET))
                    .await;
            }

            top_items.decrement(1);
            return true;
        }

        false
    }

    pub async fn get_cook_progress(&self) -> f32 {
        let current = self.cooking_time_spent.load(Ordering::Relaxed) as i32;
        let total = self.cooking_total_time.load(Ordering::Relaxed) as i32;

        if total != 0 && current != 0 {
            (current as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub async fn get_fuel_progress(&self) -> f32 {
        let remaining = self.lit_time_remaining.load(Ordering::Relaxed) as i32;
        let total = self.lit_total_time.load(Ordering::Relaxed) as i32;
        let adjusted_total = if total == 0 { 200 } else { total };

        (remaining as f32 / adjusted_total as f32).clamp(0.0, 1.0)
    }
}

#[async_trait]
impl BlockEntity for FurnaceBlockEntity {
    async fn tick(&self, world: &Arc<dyn SimpleWorld>) {
        let is_burning = self.is_burning();
        let mut is_dirty = false;
        if self.is_burning() {
            self.lit_time_remaining.fetch_sub(1, Ordering::Relaxed);
        }

        let top_items = self.items[0].lock().await;
        let is_top_items_empty = top_items.is_empty();

        let furnace_recipe = Self::get_furnace_cooking_recipe(top_items.item);
        drop(top_items);

        let can_accepet_output = self
            .can_accept_recipe_output(furnace_recipe, self.get_max_count_per_stack())
            .await;

        let mut bottom_items = self.items[1].lock().await;

        if self.is_burning() || !bottom_items.is_empty() && !is_top_items_empty {
            if !self.is_burning() && can_accepet_output {
                let fuel_ticks = get_item_burn_ticks(bottom_items.item.id).unwrap_or(0);
                self.lit_time_remaining.store(fuel_ticks, Ordering::Relaxed);
                self.lit_total_time.store(fuel_ticks, Ordering::Relaxed);
                if self.is_burning() {
                    is_dirty = true;
                    if !bottom_items.is_empty() {
                        bottom_items.decrement(1);
                        //TODO: add getRecipeRemainder().
                        //Using a lava bucket as fuel leaves an empty bucket behind.
                    }
                }
            }

            drop(bottom_items);

            if self.is_burning() && can_accepet_output {
                self.cooking_time_spent.fetch_add(1, Ordering::Relaxed);

                if self.cooking_time_spent.load(Ordering::Relaxed)
                    == self.cooking_total_time.load(Ordering::Relaxed)
                {
                    self.cooking_time_spent.store(0, Ordering::Relaxed);
                    if let Some(cooking_recipe) = furnace_recipe {
                        let cooking_total_time = cooking_recipe.cookingtime;
                        self.cooking_total_time
                            .store(cooking_total_time as u16, Ordering::Relaxed);

                        self.craft_recipe(Some(cooking_recipe)).await;
                        is_dirty = true;
                    }
                }
            } else {
                self.cooking_time_spent.store(0, Ordering::Relaxed);
            }
        } else if !self.is_burning() && self.cooking_time_spent.load(Ordering::Relaxed) > 0 {
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
                world.get_block_and_block_state(&self.position).await;
            let mut props =
                FurnaceLikeProperties::from_state_id(furnace_block_state.id, furnace_block);

            if self.is_burning() {
                props.lit = true;
                world
                    .set_block_state(
                        &self.position,
                        props.to_state_id(furnace_block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            } else {
                world
                    .set_block_state(
                        &self.position,
                        props.to_state_id(furnace_block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }

        if is_dirty {
            self.is_dirty();
        }
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

        let furnace = Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            cooking_total_time,
            cooking_time_spent,
            lit_total_time,
            lit_time_remaining,
        };
        furnace.read_data(nbt, &furnace.items);

        furnace
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_short(
            "cooking_total_time",
            self.cooking_total_time.load(Ordering::Relaxed) as i16,
        );
        nbt.put_short(
            "cooking_time_spent",
            self.cooking_time_spent.load(Ordering::Relaxed) as i16,
        );
        nbt.put_short(
            "lit_total_time",
            self.lit_total_time.load(Ordering::Relaxed) as i16,
        );
        nbt.put_short(
            "lit_time_remaining",
            self.lit_time_remaining.load(Ordering::Relaxed) as i16,
        );
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        // self.clear().await;
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FurnaceBlockEntity {
    pub const ID: &'static str = "minecraft:furnace";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            cooking_total_time: AtomicU16::new(0),
            cooking_time_spent: AtomicU16::new(0),
            lit_total_time: AtomicU16::new(0),
            lit_time_remaining: AtomicU16::new(0),
        }
    }
}

#[async_trait]
impl Inventory for FurnaceBlockEntity {
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
        let mut removed = ItemStack::EMPTY;
        let mut guard = self.items[slot].lock().await;
        std::mem::swap(&mut removed, &mut *guard);
        removed
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        split_stack(&self.items, slot, amount).await
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        let furnace_stack = self.get_stack(slot).await;
        let mut furnace_stack = furnace_stack.lock().await;

        let is_same_item =
            !stack.is_empty() && ItemStack::are_items_and_components_equal(&furnace_stack, &stack);

        *furnace_stack = stack;
        drop(furnace_stack);

        if slot == 0 && !is_same_item {
            if let Some(recipe) = Self::get_furnace_cooking_recipe(stack.item) {
                self.cooking_total_time
                    .store(recipe.cookingtime as u16, Ordering::Relaxed);
            } else {
                self.cooking_total_time.store(0, Ordering::Relaxed);
            }
            self.cooking_time_spent.store(0, Ordering::Relaxed);
            self.mark_dirty();
        }
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl Clearable for FurnaceBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}
