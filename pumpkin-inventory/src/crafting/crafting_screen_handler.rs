use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
};
use crate::slot::{NormalSlot, Slot};
use async_trait::async_trait;
use crossbeam_utils::atomic::AtomicCell;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING, RecipeResultStruct};
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::Tagable;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

/// CraftingResultSlot.java
///
/// Note: This implementation is different from the original Minecraft code.
/// Particularly, it does not have a 'result' inventory, we directly store it in the slot.
/// This slot should be never modified outside. any modifications to it make change in its input.
#[derive(Debug)]
pub struct ResultSlot {
    pub inventory: Arc<dyn RecipeInputInventory>,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
    recipe_cache: AtomicCell<Option<&'static CraftingRecipeTypes>>,
}

fn is_symmetrical_horizontally(pattern: &'static [&'static str]) -> bool {
    let width = pattern.first().map_or(0, |s| s.len());
    for row in pattern {
        if row.len() != width {
            return false; // All rows must have the same length
        }
        for j in 0..width / 2 {
            if row.chars().nth(j) != row.chars().nth(width - j - 1) {
                return false; // Characters must match symmetrically
            }
        }
    }
    true
}

async fn recipe_matches<'a>(
    recipe: &'static CraftingRecipeTypes,
    input_height: usize,
    input_width: usize,
    top_x: usize,
    top_y: usize,
    count: usize,
    inventory: &'a dyn RecipeInputInventory,
) -> Option<&'a RecipeResultStruct> {
    match recipe {
        CraftingRecipeTypes::CraftingShaped {
            key,
            pattern,
            result,
            ..
        } => {
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
            'outer: for y in 0..pattern.len() {
                for x in 0..pattern[y].len() {
                    let current_key = pattern[y].chars().nth(x).unwrap();
                    let slot = inventory
                        .get_stack((y + y_offset) * inventory.get_height() + (x + x_offset))
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

            // Check for asymmetrical recipes
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

            // TODO: Apply components
            if matched { Some(result) } else { None }
        }
        CraftingRecipeTypes::CraftingShapeless {
            ingredients,
            result,
            ..
        } => {
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

            // TODO: Apply components
            Some(result)
        }
        CraftingRecipeTypes::CraftingTransmute {
            input,
            material,
            result,
            ..
        } => {
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

            // TODO: Copy components
            Some(result)
        }
        CraftingRecipeTypes::CraftingDecoratedPot { .. } => {
            if count != 4 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }

            for position in (1..=7).step_by(2) {
                let slot = inventory.get_stack(position).await;
                let slot = slot.lock().await;

                if slot.is_empty()
                    || !slot
                        .item
                        .is_tagged_with("#minecraft:decorated_pot_ingredients")
                        .unwrap()
                {
                    return None;
                }
            }

            // TODO: Handle side textures
            Some(&RecipeResultStruct {
                id: "minecraft:decorated_pot",
                count: 1,
            })
        }
        CraftingRecipeTypes::CraftingSpecial => None,
    }
}

impl ResultSlot {
    fn stat_crafted(&self, _crafted_amount: u8, _player: &dyn InventoryPlayer) {}

    pub fn new(inventory: Arc<dyn RecipeInputInventory>) -> Self {
        Self {
            inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY)),
            recipe_cache: AtomicCell::new(None),
        }
    }

    /// Matches the recipe in the crafting inventory and returns the result.
    ///
    /// If no recipe matches, returns `None`.
    async fn match_recipe(&self) -> Option<(&RecipeResultStruct, &'static CraftingRecipeTypes)> {
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

        if let Some(cached_recipe) = self.recipe_cache.load() {
            if let Some(result) = recipe_matches(
                cached_recipe,
                input_height,
                input_width,
                top_x,
                top_y,
                count,
                &*self.inventory,
            )
            .await
            {
                return Some((result, cached_recipe));
            }
        }

        for recipe in RECIPES_CRAFTING {
            if let Some(result) = recipe_matches(
                recipe,
                input_height,
                input_width,
                top_x,
                top_y,
                count,
                &*self.inventory,
            )
            .await
            {
                self.recipe_cache.store(Some(recipe));
                return Some((result, recipe));
            }
        }

        None
    }

    async fn refill_output(&self) -> ItemStack {
        let result = self
            .match_recipe()
            .await
            .map(|x| ItemStack::from(x.0))
            .unwrap_or(ItemStack::EMPTY);
        *self.result.lock().await = result;
        result
    }
}

#[async_trait]
impl Slot for ResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        999 // this slot does not belong to any inventory
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn on_quick_move_crafted(&self, _stack: ItemStack, _stack_prev: ItemStack) {
        // refill the result slot with the recipe result
        self.refill_output().await;
    }

    async fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i).await;
            let mut stack = slot.lock().await;
            if !stack.is_empty() {
                //TODO: Handle remaining items.
                stack.item_count -= 1;
            }
        }
        self.stat_crafted(stack.item_count, player);
        self.mark_dirty().await;
    }

    async fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    async fn get_stack(&self) -> Arc<Mutex<ItemStack>> {
        self.result.clone()
    }

    async fn get_cloned_stack(&self) -> ItemStack {
        *self.result.lock().await
    }

    async fn has_stack(&self) -> bool {
        !self.result.lock().await.is_empty()
    }

    async fn set_stack(&self, _stack: ItemStack) {
        self.refill_output().await;
    }

    async fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {
        self.refill_output().await;
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }

    async fn get_max_item_count(&self) -> u8 {
        let mut count = u8::MAX;
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                count = count.min(slot.item_count);
            }
        }
        count
    }

    async fn take_stack(&self, _amount: u8) -> ItemStack {
        if self.has_stack().await {
            let stack = self.result.lock().await;
            // Vanilla: net.minecraft.world.inventory.ResultContainer#removeItem
            // Regardless of the amount, we always return the full stack
            *stack
        } else {
            ItemStack::EMPTY
        }
    }
}

#[async_trait]
impl ScreenHandlerListener for ResultSlot {
    async fn on_slot_update(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        slot: u8,
        _stack: ItemStack,
    ) {
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
    }
}

// AbstractCraftingScreenHandler.java
#[async_trait]
pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    async fn add_recipe_slots(&mut self, crafing_inventory: Arc<dyn RecipeInputInventory>) {
        let result_slot = Arc::new(ResultSlot::new(crafing_inventory.clone()));
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
    }
}

// CraftingMenu
pub struct CraftingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl CraftingTableScreenHandler {
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let crafting_inventory: Arc<dyn RecipeInputInventory> =
            Arc::new(CraftingInventory::new(3, 3));

        let mut crafting_table_handler = CraftingTableScreenHandler {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Crafting)),
            crafting_inventory: crafting_inventory.clone(),
        };

        crafting_table_handler
            .add_recipe_slots(crafting_inventory)
            .await;

        // Add player inventory slots
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        crafting_table_handler.add_player_slots(&player_inventory);

        crafting_table_handler
    }
}

impl RecipeFinderScreenHandler for CraftingTableScreenHandler {}

#[async_trait]
impl ScreenHandler for CraftingTableScreenHandler {
    async fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player).await;
        //TODO: this.craftingResultInventory.clear();
        self.drop_inventory(player, self.crafting_inventory.clone())
            .await;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    async fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack().await {
            let slot_stack = slot.get_stack().await;
            let mut slot_stack = slot_stack.lock().await;
            let stack_prev = *slot_stack;

            if slot_index == 0 {
                // From crafting result slot - move to player inventory (slots 10-46)
                if !self.insert_item(&mut slot_stack, 10, 46, true).await {
                    return ItemStack::EMPTY;
                }
            } else if (1..=9).contains(&slot_index) {
                // From crafting input slots - try to move to player inventory (slots 10-46)
                if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                    return ItemStack::EMPTY;
                }
            } else if (10..46).contains(&slot_index) {
                // From player inventory - try to move to crafting input slots first (1-9)
                if !self.insert_item(&mut slot_stack, 1, 10, false).await {
                    // If that fails, try moving within player inventory
                    if slot_index < 37 {
                        // From main inventory to hotbar
                        if !self.insert_item(&mut slot_stack, 37, 46, false).await {
                            return ItemStack::EMPTY;
                        }
                    } else {
                        // From hotbar to main inventory
                        if !self.insert_item(&mut slot_stack, 10, 37, false).await {
                            return ItemStack::EMPTY;
                        }
                    }
                }
            } else {
                // Any other slot - try to move to player inventory
                if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                    return ItemStack::EMPTY;
                }
            }

            let stack = *slot_stack;
            drop(slot_stack); // release the lock before calling other methods

            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY, stack_prev).await;
            } else {
                slot.mark_dirty().await;
            }

            if stack.item_count == stack_prev.item_count {
                // Nothing changed
                return ItemStack::EMPTY;
            }

            slot.on_take_item(player, &stack).await;

            if slot_index == 0 {
                slot.on_quick_move_crafted(stack, stack_prev).await;
                // For crafting result slot, drop any remaining items
                if !stack.is_empty() {
                    player.drop_item(stack, false).await;
                }
            }

            return stack_prev;
        }

        ItemStack::EMPTY
    }
}

impl CraftingScreenHandler<CraftingInventory> for CraftingTableScreenHandler {}
