use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_world::{inventory::split_stack, item::ItemStack};
use tokio::sync::Mutex;

use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING, RecipeResultStruct};
use pumpkin_data::tag::Tagable;
use pumpkin_world::inventory::{Clearable, Inventory};

use super::recipes::RecipeInputInventory;

#[derive(Debug, Clone)]
pub struct CraftingInventory {
    pub width: u8,
    pub height: u8,
    pub items: Vec<Arc<Mutex<ItemStack>>>,
}

impl CraftingInventory {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            items: {
                // Creates a Vec with different Mutexes for each slot
                let mut v = Vec::with_capacity(width as usize * height as usize);
                (0..width as usize * height as usize)
                    .for_each(|_| v.push(Arc::new(Mutex::new(ItemStack::EMPTY))));
                v
            },
        }
    }
}

#[async_trait]
impl Inventory for CraftingInventory {
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
        *self.items[slot].lock().await = stack;
    }
}

#[async_trait]
impl Clearable for CraftingInventory {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}

#[async_trait]
impl RecipeInputInventory for CraftingInventory {
    fn get_width(&self) -> usize {
        self.width as usize
    }

    fn get_height(&self) -> usize {
        self.height as usize
    }

    async fn match_recipe(&self) -> Option<&RecipeResultStruct> {
        let mut count: usize = 0;

        for i in 0..self.size() {
            let slot = self.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                count += 1;
            }
        }

        'next_recipe: for recipe in RECIPES_CRAFTING {
            match recipe {
                CraftingRecipeTypes::CraftingShaped {
                    key,
                    pattern,
                    result,
                    ..
                } => {
                    if pattern.len() > self.get_height()
                        || pattern.first().unwrap().len() > self.get_width()
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

                    for y_offset in 0..=(self.get_height() - pattern.len()) {
                        'next_offset: for x_offset in 0..=(self.get_width() - pattern[0].len()) {
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
                                        .get_stack(
                                            (y + y_offset) * self.get_height() + (x + x_offset),
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
                    'next_slot: for i in 0..self.size() {
                        let slot = self.get_stack(i).await;
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

                    'item_stack: for i in 0..self.size() {
                        let slot = self.get_stack(i).await;
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
                    if count != 4 || self.get_width() != 3 || self.get_height() != 3 {
                        continue 'next_recipe;
                    }

                    for position in (1..=7).step_by(2) {
                        let slot = self.get_stack(position).await;
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
