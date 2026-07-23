use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    offer_or_drop_stack,
};
use crate::slot::{BoxFuture, NormalSlot, Slot};
use crate::window_property::{Stonecutter, WindowProperty};

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{RECIPES_STONECUTTING, StonecutterRecipe};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

/// Vanilla `StonecutterMenu` uses -1 for "no recipe selected".
const NO_RECIPE: i32 = -1;

pub struct StonecutterScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    /// Selected recipe index, or [`NO_RECIPE`].
    pub selected_recipe: AtomicI32,
}

impl StonecutterScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Stonecutter));
        let input_inventory = Arc::new(SimpleInventory::new(1));
        let output_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
            selected_recipe: AtomicI32::new(NO_RECIPE),
        };

        handler.add_slot(Arc::new(NormalSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            0,
        )));
        handler.add_slot(Arc::new(StonecutterOutputSlot::new(
            output_inventory as Arc<dyn Inventory>,
            input_inventory as Arc<dyn Inventory>,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();

        handler.add_player_slots(&player_inventory);

        handler
    }

    async fn send_selected_recipe_property(&self) {
        let Some(sync_handler) = self.behaviour.sync_handler.as_ref() else {
            return;
        };
        let selected = self.selected_recipe.load(Ordering::Relaxed);
        let (id, val) =
            WindowProperty::new(Stonecutter::SelectedRecipe, selected as i16).into_tuple();
        sync_handler
            .update_property(&self.behaviour, i32::from(id), i32::from(val))
            .await;
    }

    async fn update_output(&self) {
        let input_stack = self.input_inventory.get_stack(0).await;
        let input_lock = input_stack.lock().await;

        if input_lock.is_empty() {
            self.output_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
            self.selected_recipe.store(NO_RECIPE, Ordering::Relaxed);
            drop(input_lock);
            self.send_selected_recipe_property().await;
            return;
        }

        let available_recipes = Self::get_available_recipes(&input_lock);
        let recipe_index = self.selected_recipe.load(Ordering::Relaxed);

        if recipe_index >= 0 && (recipe_index as usize) < available_recipes.len() {
            let recipe = available_recipes[recipe_index as usize];
            if let Some(item) = Item::from_registry_key(recipe.result.id) {
                let result = ItemStack::new(recipe.result.count, item);
                self.output_inventory.set_stack(0, result).await;
            } else {
                // Bad recipe data must not panic the network thread.
                self.output_inventory
                    .set_stack(0, ItemStack::EMPTY.clone())
                    .await;
                self.selected_recipe.store(NO_RECIPE, Ordering::Relaxed);
            }
        } else {
            // Clear invalid selection when input changes.
            if recipe_index != NO_RECIPE
                && (recipe_index < 0 || (recipe_index as usize) >= available_recipes.len())
            {
                self.selected_recipe.store(NO_RECIPE, Ordering::Relaxed);
            }
            self.output_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
        }
        drop(input_lock);
        self.send_selected_recipe_property().await;
    }

    fn get_available_recipes(input: &ItemStack) -> Vec<&'static StonecutterRecipe> {
        let item = input.item;
        RECIPES_STONECUTTING
            .iter()
            .filter(|r| r.ingredient.match_item(item))
            .collect()
    }
}

impl ScreenHandler for StonecutterScreenHandler {
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            // Input changed or output taken — refresh result + selected-recipe property.
            if slot_index == 0 || slot_index == 1 {
                self.update_output().await;
            }
        })
    }

    fn on_button_click<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        button_id: i32,
    ) -> ScreenHandlerFuture<'a, bool> {
        // Vanilla: button id is the recipe index in the available list for current input.
        Box::pin(async move {
            if button_id < 0 {
                return false;
            }
            let input_stack = self.input_inventory.get_stack(0).await;
            let input_lock = input_stack.lock().await;
            if input_lock.is_empty() {
                return false;
            }
            let available = Self::get_available_recipes(&input_lock);
            drop(input_lock);
            if (button_id as usize) >= available.len() {
                return false;
            }
            self.selected_recipe.store(button_id, Ordering::Relaxed);
            self.update_output().await;
            true
        })
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // Return leftover input to the player (output is craft-only, never stored).
            let stack = self.input_inventory.remove_stack(0).await;
            if !stack.is_empty() {
                offer_or_drop_stack(player, stack).await;
            }
            self.output_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
            self.selected_recipe.store(NO_RECIPE, Ordering::Relaxed);
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ScreenHandlerFuture<'a, ItemStack> {
        Box::pin(async move {
            let mut stack = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

            if let Some(slot) = slot {
                let mut slot_stack = slot.get_cloned_stack().await;
                if !slot_stack.is_empty() {
                    stack = slot_stack.clone();
                    if slot_index < 2 {
                        // From Stonecutter to Player
                        if !self.insert_item(&mut slot_stack, 2, 38, true).await {
                            return ItemStack::EMPTY.clone();
                        }
                        slot.on_quick_move_crafted(slot_stack.clone(), stack.clone())
                            .await;
                    } else {
                        // From Player to Stonecutter input slot (0)
                        if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }

                    if slot_stack.is_empty() {
                        slot.set_stack(ItemStack::EMPTY.clone()).await;
                    } else {
                        slot.set_stack(slot_stack).await;
                    }

                    if slot_index == 0 || slot_index == 1 {
                        self.update_output().await;
                    }
                }
            }
            stack
        })
    }
}

pub struct StonecutterOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub input_inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

use std::sync::atomic::AtomicU8;

impl StonecutterOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        input_inventory: Arc<dyn Inventory>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            input_inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for StonecutterOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
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
            let input_stack = self.input_inventory.get_stack(0).await;
            let mut input_lock = input_stack.lock().await;
            if !input_lock.is_empty() {
                input_lock.item_count -= 1;
                if input_lock.item_count == 0 {
                    *input_lock = ItemStack::EMPTY.clone();
                }
            }
            self.mark_dirty().await;
        })
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.inventory.get_stack(self.index).await })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            stack.lock().await.clone()
        })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            !stack.lock().await.is_empty()
        })
    }

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            // Do nothing
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}
