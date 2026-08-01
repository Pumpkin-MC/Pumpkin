use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use crate::slot::{BoxFuture, NormalSlot, Slot};

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{RECIPES_STONECUTTING, StonecutterRecipe};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

pub struct StonecutterScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub selected_recipe: AtomicU8,
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
            selected_recipe: AtomicU8::new(u8::MAX),
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

    async fn update_output(&self) {
        let input_stack = self.input_inventory.get_stack(0).await;
        let input_lock = input_stack.lock().await;

        if input_lock.is_empty() {
            self.output_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
            self.selected_recipe.store(u8::MAX, Ordering::Relaxed);
            return;
        }

        let available_recipes = Self::get_available_recipes(&input_lock);
        let recipe_index = self.selected_recipe.load(Ordering::Relaxed);

        if recipe_index != u8::MAX && (recipe_index as usize) < available_recipes.len() {
            let recipe = available_recipes[recipe_index as usize];
            let item =
                Item::from_registry_key(recipe.result.id).expect("Invalid recipe result item");
            let result = ItemStack::new(recipe.result.count, item);
            self.output_inventory.set_stack(0, result).await;
        } else {
            self.output_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
        }
    }

    async fn select_recipe(&self, id: i32) -> bool {
        if i32::from(self.selected_recipe.load(Ordering::Relaxed)) == id {
            return false;
        }

        let input_stack = self.input_inventory.get_stack(0).await;
        let input_stack = input_stack.lock().await;
        let recipe_count = Self::get_available_recipes(&input_stack).len();
        drop(input_stack);

        if !(0..recipe_count).contains(&(id as usize)) {
            return false;
        }

        self.selected_recipe.store(id as u8, Ordering::Relaxed);
        self.update_output().await;
        true
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
            if slot_index == 0 || slot_index == 1 {
                self.update_output().await;
            }
        })
    }

    fn on_button_click<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        id: i32,
    ) -> ScreenHandlerFuture<'a, bool> {
        Box::pin(async move {
            let clicked = self.select_recipe(id).await;
            if clicked {
                self.send_content_updates().await;
            }
            clicked
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
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
                        // From Player to Stonecutter
                        // Try input slot (0)
                        if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }

                    let moved_count = stack.item_count - slot_stack.item_count;
                    if slot_stack.is_empty() {
                        slot.set_stack(ItemStack::EMPTY.clone()).await;
                    } else {
                        slot.set_stack(slot_stack).await;
                    }

                    if slot_index == 1 {
                        let mut taken_stack = stack.clone();
                        taken_stack.set_count(moved_count);
                        slot.on_take_item(player, &taken_stack).await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_equipment_slots, entity_equipment::EntityEquipment};

    fn handler() -> StonecutterScreenHandler {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        StonecutterScreenHandler::new(0, &player_inventory)
    }

    #[tokio::test]
    async fn selecting_a_valid_recipe_refreshes_the_output() {
        let handler = handler();
        let input = ItemStack::new(1, &Item::STONE);
        let recipes = StonecutterScreenHandler::get_available_recipes(&input);
        assert!(!recipes.is_empty());
        handler.input_inventory.set_stack(0, input).await;

        assert!(handler.select_recipe(0).await);
        assert_eq!(handler.selected_recipe.load(Ordering::Relaxed), 0);

        let output = handler.output_inventory.get_stack(0).await;
        let output = output.lock().await;
        let expected = Item::from_registry_key(recipes[0].result.id)
            .expect("stonecutter recipe result must be a registered item");
        assert_eq!(output.item.id, expected.id);
        assert_eq!(output.item_count, recipes[0].result.count);
    }

    #[tokio::test]
    async fn invalid_recipe_selection_preserves_the_current_output() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(0, ItemStack::new(1, &Item::STONE))
            .await;
        assert!(handler.select_recipe(0).await);

        let output_before = handler.output_inventory.remove_stack(0).await;
        handler
            .output_inventory
            .set_stack(0, output_before.clone())
            .await;

        assert!(!handler.select_recipe(i32::MAX).await);
        assert_eq!(handler.selected_recipe.load(Ordering::Relaxed), 0);
        let output_after = handler.output_inventory.remove_stack(0).await;
        assert_eq!(output_after.item.id, output_before.item.id);
        assert_eq!(output_after.item_count, output_before.item_count);
    }

    #[tokio::test]
    async fn output_refresh_clears_selection_when_input_is_depleted() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(0, ItemStack::new(1, &Item::STONE))
            .await;
        assert!(handler.select_recipe(0).await);

        handler
            .input_inventory
            .set_stack(0, ItemStack::EMPTY.clone())
            .await;
        handler.update_output().await;

        assert_eq!(handler.selected_recipe.load(Ordering::Relaxed), u8::MAX);
        let output = handler.output_inventory.get_stack(0).await;
        assert!(output.lock().await.is_empty());
    }
}
