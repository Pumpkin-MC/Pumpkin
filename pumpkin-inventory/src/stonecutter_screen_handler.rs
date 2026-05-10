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
            if slot_index == 0 {
                self.update_output().await;
            }
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
                        // From Player to Stonecutter
                        // Try input slot (0)
                        if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }

                    if slot_stack.is_empty() {
                        slot.set_stack(ItemStack::EMPTY.clone()).await;
                    } else {
                        slot.set_stack(slot_stack).await;
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
        _player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
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
