use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use async_trait::async_trait;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING, RecipeResultStruct};
use pumpkin_data::tag::Tagable;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
};
use crate::slot::{NormalSlot, Slot};

use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};

// TODO: Implement ResultSlot
// CraftingResultSlot.java
#[derive(Debug)]
pub struct ResultSlot {
    pub inventory: Arc<dyn RecipeInputInventory>,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
}

impl ResultSlot {
    fn stat_crafted(&self, _crafted_amount: u8, _player: &dyn InventoryPlayer) {}

    pub fn new(inventory: Arc<dyn RecipeInputInventory>) -> Self {
        Self {
            inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY)),
        }
    }

    async fn match_recipe(&self) -> Option<&RecipeResultStruct> {
        let mut count: usize = 0;
        let inventory_width = self.inventory.get_width();
        let mut top_x = 9;
        let mut top_y = 9;
        let mut bottom_x = 0;
        let mut bottom_y = 0;
        for i in 0..self.inventory.size() {
            let x = i % inventory_width;
            let y = i / inventory_width;
            if x < top_x {
                top_x = x;
            }
            if y < top_y {
                top_y = y;
            }
            if x > bottom_x {
                bottom_x = x;
            }
            if y > bottom_y {
                bottom_y = y;
            }

            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                count += 1;
            }
        }
        let input_width = bottom_x - top_x + 1;
        let input_height = bottom_y - top_y + 1;

        if count == 0 {
            return None;
        }

        'next_recipe: for recipe in RECIPES_CRAFTING {
            match recipe {
                CraftingRecipeTypes::CraftingShaped {
                    key,
                    pattern,
                    result,
                    ..
                } => {
                    if pattern.len() != input_height
                        || pattern.first().unwrap().len() != input_width
                    {
                        continue;
                    }

                    if count
                        != pattern
                            .iter()
                            .map(|l| l.chars().filter(|c| *c != ' ').count())
                            .sum::<usize>()
                    {
                        continue;
                    }

                    for y_offset in 0..=(self.inventory.get_height() - pattern.len()) {
                        'next_offset: for x_offset in
                            0..=(self.inventory.get_width() - pattern[0].len())
                        {
                            // Check if pattern matches
                            for y in 0..pattern.len() {
                                for x in 0..pattern[y].len() {
                                    let current_key = pattern[y].chars().nth(x).unwrap();
                                    if current_key == ' ' {
                                        continue;
                                    }

                                    let ingredient = key
                                        .iter()
                                        .find_map(|(k, v)| (*k == current_key).then_some(v))
                                        .expect("Crafting recipe used invalid key");

                                    let slot = self
                                        .inventory
                                        .get_stack(
                                            (y + y_offset) * self.inventory.get_height()
                                                + (x + x_offset),
                                        )
                                        .await;
                                    let slot = slot.lock().await;

                                    if !ingredient.match_item(slot.item) {
                                        continue 'next_offset;
                                    }
                                }
                            }

                            // TODO: Apply components
                            return Some(result);
                        }
                    }

                    continue 'next_recipe;
                }
                CraftingRecipeTypes::CraftingShapeless {
                    ingredients,
                    result,
                    ..
                } => {
                    if count != ingredients.len() {
                        continue;
                    }

                    let mut ingredient_used = vec![false; ingredients.len()];
                    'next_slot: for i in 0..self.inventory.size() {
                        let slot = self.inventory.get_stack(i).await;
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

                        continue 'next_recipe;
                    }

                    // TODO: Apply components
                    return Some(result);
                }
                CraftingRecipeTypes::CraftingTransmute {
                    input,
                    material,
                    result,
                    ..
                } => {
                    if count != 2 {
                        continue;
                    }

                    'item_stack: for i in 0..self.inventory.size() {
                        let slot = self.inventory.get_stack(i).await;
                        let slot = slot.lock().await;

                        if slot.is_empty() {
                            continue 'item_stack;
                        }

                        if !material.match_item(slot.item) && !input.match_item(slot.item) {
                            continue 'next_recipe;
                        }
                    }

                    // TODO: Copy components
                    return Some(result);
                }
                CraftingRecipeTypes::CraftingDecoratedPot { .. } => {
                    if count != 4
                        || self.inventory.get_width() != 3
                        || self.inventory.get_height() != 3
                    {
                        continue 'next_recipe;
                    }

                    for position in (1..=7).step_by(2) {
                        let slot = self.inventory.get_stack(position).await;
                        let slot = slot.lock().await;

                        if slot.is_empty()
                            || !slot
                                .item
                                .is_tagged_with("#minecraft:decorated_pot_ingredients")
                                .unwrap()
                        {
                            continue 'next_recipe;
                        }
                    }

                    // TODO: Handle side textures
                    return Some(&RecipeResultStruct {
                        id: "minecraft:decorated_pot",
                        count: 1,
                    });
                }
                CraftingRecipeTypes::CraftingSpecial => continue,
            }
        }

        None
    }
}

#[async_trait]
impl Slot for ResultSlot {
    async fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
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

    async fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        // Vanilla does not have this check, but it is a good idea to have it
        debug_assert_eq!(stack.item_count, self.get_cloned_stack().await.item_count);
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

    async fn has_stack(&self) -> bool {
        !self.result.lock().await.is_empty()
    }

    async fn get_stack(&self) -> Arc<Mutex<ItemStack>> {
        self.result.clone()
    }

    async fn get_cloned_stack(&self) -> ItemStack {
        *self.result.lock().await
    }

    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        0
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
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
            let result = self
                .match_recipe()
                .await
                .map(ItemStack::from)
                .unwrap_or(ItemStack::EMPTY);
            *self.result.lock().await = result;

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
