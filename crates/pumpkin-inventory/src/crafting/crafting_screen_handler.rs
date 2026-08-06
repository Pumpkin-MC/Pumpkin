//! Crafting screen handler implementation.
//!
//! This module provides screen handlers for crafting mechanics:
//! - [`CraftingScreenHandler`] - Trait for crafting screen handlers
//! - [`CraftingTableScreenHandler`] - The 3x3 crafting table UI
//! - [`ResultSlot`] - The special result slot that shows crafted items
//!
//! # Recipe Matching
//!
//! Crafting recipes are matched against the items in the crafting grid.
//! The system supports:
//! - Shaped recipes (specific patterns)
//! - Shapeless recipes (any arrangement)
//! - Transmute recipes (upgrading items)
//! - Special recipes (like decorated pots)

use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use super::recipe_provider::{GenericRecipe, RecipeProvider};
use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    ScreenHandlerListener,
};
use crate::slot::{BoxFuture, NormalSlot, Slot};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::codec::recipe::{DynamicRecipe, OwnedCraftingRecipe};
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

/// The result slot in a crafting screen.
pub struct ResultSlot {
    /// The crafting inventory (grid) that provides recipe input.
    pub inventory: Arc<dyn RecipeInputInventory>,
    /// Protocol ID for this slot (assigned by screen handler).
    pub id: AtomicU8,
    /// The cached result item stack.
    pub result: Arc<Mutex<ItemStack>>,
    /// Provider for dynamic recipes.
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
}

pub struct RecipeResult {
    pub item_id: String,
    pub count: u8,
}

/// Checks if a recipe pattern is symmetrical horizontally.
fn is_symmetrical_horizontally(pattern: &[&str]) -> bool {
    let width = pattern.first().map_or(0, |s| s.len());
    for row in pattern {
        if row.len() != width {
            return false;
        }
        for j in 0..width / 2 {
            if row.chars().nth(j) != row.chars().nth(width - j - 1) {
                return false;
            }
        }
    }
    true
}

/// Checks if a crafting recipe matches the current inventory state.
#[expect(clippy::too_many_lines)]
async fn recipe_matches(
    recipe: GenericRecipe<'_>,
    input_height: usize,
    input_width: usize,
    top_x: usize,
    top_y: usize,
    count: usize,
    inventory: &dyn RecipeInputInventory,
) -> Option<RecipeResult> {
    match recipe {
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingShaped {
            key,
            pattern,
            result,
            ..
        }) => {
            if pattern.len() != input_height || pattern.first().unwrap().len() != input_width {
                return None;
            }

            if count
                != pattern
                    .iter()
                    .map(|l| l.chars().filter(|c| *c != ' ').count())
                    .sum::<usize>()
            {
                return None;
            }

            let x_offset = top_x;
            let y_offset = top_y;

            let mut matched = true;
            'outer: for (y, row_str) in pattern.iter().enumerate() {
                for (x, current_key) in row_str.chars().enumerate() {
                    let slot = inventory
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset))
                        .await;
                    if current_key == ' ' {
                        if !slot.lock().await.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }

                    let ingredient = key
                        .iter()
                        .find_map(|(k, v)| (*k == current_key).then_some(v))
                        .expect("Crafting recipe used invalid key");

                    let slot = slot.lock().await;

                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }

            if !matched && !is_symmetrical_horizontally(pattern) {
                matched = true;
                'outer: for y in 0..pattern.len() {
                    for x in 0..pattern[y].len() {
                        let current_key = pattern[y].chars().nth(x).unwrap();
                        let slot = inventory
                            .get_stack(
                                (y + y_offset) * inventory.get_height()
                                    + (x_offset + input_width - 1 - x),
                            )
                            .await;
                        if current_key == ' ' {
                            if !slot.lock().await.is_empty() {
                                matched = false;
                                break 'outer;
                            }
                            continue;
                        }
                        let ingredient = key
                            .iter()
                            .find_map(|(k, v)| (*k == current_key).then_some(v))
                            .expect("Crafting recipe used invalid key");
                        let slot = slot.lock().await;
                        if !ingredient.match_item(slot.item) {
                            matched = false;
                            break 'outer;
                        }
                    }
                }
            }

            matched.then_some(RecipeResult {
                item_id: result.id.to_string(),
                count: result.count,
            })
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingShapeless {
            ingredients,
            result,
            ..
        }) => {
            if count != ingredients.len() {
                return None;
            }
            let mut ingredient_used = vec![false; ingredients.len()];
            'next_slot: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'next_slot;
                }
                for i in 0..ingredients.len() {
                    if !ingredient_used[i] && ingredients[i].match_item(slot.item) {
                        ingredient_used[i] = true;
                        continue 'next_slot;
                    }
                }
                return None;
            }
            Some(RecipeResult {
                item_id: result.id.to_string(),
                count: result.count,
            })
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingTransmute {
            input,
            material,
            result,
            ..
        }) => {
            if count != 2 {
                return None;
            }
            'item_stack: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'item_stack;
                }
                if !material.match_item(slot.item) && !input.match_item(slot.item) {
                    return None;
                }
            }
            Some(RecipeResult {
                item_id: result.id.to_string(),
                count: result.count,
            })
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingDecoratedPot { .. }) => {
            if count != 4 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            for position in (1..=7).step_by(2) {
                let slot = inventory.get_stack(position).await;
                let slot = slot.lock().await;
                if slot.is_empty()
                    || !slot
                        .item
                        .has_tag(&tag::Item::MINECRAFT_DECORATED_POT_INGREDIENTS)
                {
                    return None;
                }
            }
            Some(RecipeResult {
                item_id: "minecraft:decorated_pot".to_string(),
                count: 1,
            })
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shaped {
            pattern,
            key,
            result,
            ..
        }) => {
            if pattern.len() != input_height || pattern.first().unwrap().len() != input_width {
                return None;
            }
            if count
                != pattern
                    .iter()
                    .map(|l| l.chars().filter(|c| *c != ' ').count())
                    .sum::<usize>()
            {
                return None;
            }
            let x_offset = top_x;
            let y_offset = top_y;
            let mut matched = true;
            'outer: for (y, row_str) in pattern.iter().enumerate() {
                for (x, current_key) in row_str.chars().enumerate() {
                    let slot = inventory
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset))
                        .await;
                    if current_key == ' ' {
                        if !slot.lock().await.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }
                    let ingredient = key
                        .iter()
                        .find(|(k, _)| *k == current_key)
                        .map(|(_, v)| v)
                        .expect("Crafting recipe used invalid key");
                    let slot = slot.lock().await;
                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }
            matched.then_some(RecipeResult {
                item_id: result.item_id.clone(),
                count: result.count,
            })
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shapeless {
            ingredients,
            result,
            ..
        }) => {
            if count != ingredients.len() {
                return None;
            }
            let mut ingredient_used = vec![false; ingredients.len()];
            'next_slot: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'next_slot;
                }
                for i in 0..ingredients.len() {
                    if !ingredient_used[i] && ingredients[i].match_item(slot.item) {
                        ingredient_used[i] = true;
                        continue 'next_slot;
                    }
                }
                return None;
            }
            Some(RecipeResult {
                item_id: result.item_id.clone(),
                count: result.count,
            })
        }
        _ => None,
    }
}

impl ResultSlot {
    pub fn new(
        inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        Self {
            inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            recipe_provider: provider,
        }
    }

    async fn match_recipe(&self) -> Option<RecipeResult> {
        let mut count: usize = 0;
        let inventory_width = self.inventory.get_width();
        let mut top_x = 9;
        let mut top_y = 9;
        let mut bottom_x = 0;
        let mut bottom_y = 0;
        for i in 0..self.inventory.size() {
            let x = i % inventory_width;
            let y = i / inventory_width;
            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                top_x = top_x.min(x);
                top_y = top_y.min(y);
                bottom_x = bottom_x.max(x);
                bottom_y = bottom_y.max(y);
                count += 1;
            }
        }
        if count == 0 {
            return None;
        }
        let input_width = bottom_x + 1 - top_x;
        let input_height = bottom_y + 1 - top_y;

        for recipe in RECIPES_CRAFTING {
            if let Some(result) = recipe_matches(
                GenericRecipe::Vanilla(recipe),
                input_height,
                input_width,
                top_x,
                top_y,
                count,
                &*self.inventory,
            )
            .await
            {
                return Some(result);
            }
        }

        if let Some(provider) = &self.recipe_provider {
            let dynamic = provider.get_dynamic_recipes().await;
            for recipe in &dynamic {
                if let DynamicRecipe::Crafting(crafting) = recipe
                    && let Some(result) = recipe_matches(
                        GenericRecipe::Dynamic(crafting),
                        input_height,
                        input_width,
                        top_x,
                        top_y,
                        count,
                        &*self.inventory,
                    )
                    .await
                {
                    return Some(result);
                }
            }
        }
        None
    }

    async fn refill_output(&self) -> ItemStack {
        let result = if let Some(matched) = self.match_recipe().await {
            let key = matched
                .item_id
                .strip_prefix("minecraft:")
                .unwrap_or(&matched.item_id);
            let item = pumpkin_data::item::Item::from_registry_key(key)
                .unwrap_or(&pumpkin_data::item::Item::AIR);
            ItemStack::new(matched.count, item)
        } else {
            ItemStack::EMPTY.clone()
        };
        *self.result.lock().await = result.clone();
        result
    }
}

impl Slot for ResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }
    fn get_index(&self) -> usize {
        999
    }
    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }
    fn on_quick_move_crafted(
        &self,
        _stack: ItemStack,
        _stack_prev: ItemStack,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            player
                .increment_stat(
                    StatisticCategory::Crafted,
                    stack.item.id as i32,
                    stack.item_count as i32,
                )
                .await;
            for i in 0..self.inventory.size() {
                let slot = self.inventory.get_stack(i).await;
                let mut stack = slot.lock().await;
                if !stack.is_empty() {
                    stack.item_count -= 1;
                }
            }
            self.mark_dirty().await;
        })
    }
    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }
    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.result.clone() })
    }
    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.result.lock().await.clone() })
    }
    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move { !self.result.lock().await.is_empty() })
    }
    fn set_stack(&self, _stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move {
            let mut count = u8::MAX;
            for i in 0..self.inventory.size() {
                let slot = self.inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if !slot.is_empty() {
                    count = count.min(slot.item_count);
                }
            }
            count
        })
    }
    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            if self.has_stack().await {
                self.result.lock().await.clone()
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }
}

impl ScreenHandlerListener for ResultSlot {
    fn on_slot_update<'a>(
        &'a self,
        screen_handler: &'a ScreenHandlerBehaviour,
        slot: u8,
        _stack: ItemStack,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if (0..=(self.inventory.get_width() * self.inventory.get_height()))
                .contains(&(slot as usize))
            {
                let result = self.refill_output().await;
                let next_revision = screen_handler.next_revision();
                if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                    sync_handler
                        .update_slot(screen_handler, 0, &result, next_revision)
                        .await;
                }
            }
        })
    }
}

pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    fn add_recipe_slots<'a>(
        &'a mut self,
        crafing_inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let result_slot = Arc::new(ResultSlot::new(crafing_inventory.clone(), provider));
            self.add_slot(result_slot.clone());
            let width = crafing_inventory.get_width();
            let height = crafing_inventory.get_height();
            for i in 0..width {
                for j in 0..height {
                    let input_slot = NormalSlot::new(crafing_inventory.clone(), j + i * width);
                    self.add_slot(Arc::new(input_slot));
                }
            }
            self.add_listener(result_slot).await;
        })
    }
}

pub struct CraftingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl CraftingTableScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        let crafting_inventory: Arc<dyn RecipeInputInventory> =
            Arc::new(CraftingInventory::new(3, 3));
        let mut crafting_table_handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Crafting)),
            crafting_inventory: crafting_inventory.clone(),
        };
        crafting_table_handler
            .add_recipe_slots(crafting_inventory, provider)
            .await;
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        crafting_table_handler.add_player_slots(&player_inventory);
        crafting_table_handler
    }
}

impl RecipeFinderScreenHandler for CraftingTableScreenHandler {}

impl ScreenHandler for CraftingTableScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }
    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.drop_inventory(player, self.crafting_inventory.clone())
                .await;
        })
    }
    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();
            if slot.has_stack().await {
                let slot_stack = slot.get_stack().await;
                let mut slot_stack = slot_stack.lock().await;
                let stack_prev = slot_stack.clone();
                if slot_index == 0 {
                    if !self.insert_item(&mut slot_stack, 10, 46, true).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (1..=9).contains(&slot_index) {
                    if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (10..46).contains(&slot_index) {
                    if !self.insert_item(&mut slot_stack, 1, 10, false).await {
                        if slot_index < 37 {
                            if !self.insert_item(&mut slot_stack, 37, 46, false).await {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if !self.insert_item(&mut slot_stack, 10, 37, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                } else if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                    return ItemStack::EMPTY.clone();
                }
                let stack = slot_stack.clone();
                drop(slot_stack);
                if stack.is_empty() {
                    slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone())
                        .await;
                } else {
                    slot.mark_dirty().await;
                }
                if stack.item_count == stack_prev.item_count {
                    return ItemStack::EMPTY.clone();
                }

                let mut taken_stack = stack_prev.clone();
                taken_stack.set_count(stack_prev.item_count - stack.item_count);
                slot.on_take_item(player, &taken_stack).await;

                if slot_index == 0 {
                    slot.on_quick_move_crafted(stack.clone(), stack_prev.clone())
                        .await;
                    if !stack.is_empty() {
                        player.drop_item(stack, false).await;
                    }
                }
                return stack_prev;
            }
            ItemStack::EMPTY.clone()
        })
    }
}

impl CraftingScreenHandler<CraftingInventory> for CraftingTableScreenHandler {}
