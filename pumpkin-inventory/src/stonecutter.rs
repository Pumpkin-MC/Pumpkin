use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::item::Item;
use pumpkin_data::recipes::{StonecuttingRecipe, RECIPES_STONECUTTING};
use pumpkin_data::screen::WindowType;
use pumpkin_world::{
    inventory::{Clearable, Inventory, InventoryFuture, split_stack},
    item::ItemStack,
};
use tokio::sync::Mutex;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture,
    },
    slot::{BoxFuture, NormalSlot, Slot},
};

use std::sync::atomic::{AtomicU8, Ordering};

/// Returns all stonecutting recipes whose ingredient matches the given item.
#[must_use]
pub fn get_stonecutting_recipes_for(
    item: &Item,
) -> Vec<&'static StonecuttingRecipe> {
    RECIPES_STONECUTTING
        .iter()
        .filter(|recipe| recipe.ingredient.match_item(item))
        .collect()
}

/// A simple inventory backing the stonecutter's input slot.
pub struct StonecutterInventory {
    pub items: [Arc<Mutex<ItemStack>>; 1],
}

impl Default for StonecutterInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl StonecutterInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: [Arc::new(Mutex::new(ItemStack::EMPTY.clone()))],
        }
    }
}

impl Inventory for StonecutterInventory {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move { self.items[0].lock().await.is_empty() })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for StonecutterInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            *self.items[0].lock().await = ItemStack::EMPTY.clone();
        })
    }
}

/// Output slot for the stonecutter. Consumes one item from input when taken.
pub struct StonecutterOutputSlot {
    pub input_inventory: Arc<StonecutterInventory>,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
}

impl StonecutterOutputSlot {
    #[must_use]
    pub fn new(input_inventory: Arc<StonecutterInventory>) -> Self {
        Self {
            input_inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for StonecutterOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.input_inventory.clone()
    }

    fn get_index(&self) -> usize {
        999 // Output slot does not belong to the backing inventory
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
            // Consume one item from the input slot
            let input_stack = self.input_inventory.get_stack(0).await;
            let mut guard = input_stack.lock().await;
            if !guard.is_empty() {
                guard.item_count -= 1;
            }
            drop(guard);
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

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            *self.result.lock().await = stack;
        })
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            *self.result.lock().await = stack;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.input_inventory.mark_dirty();
        })
    }

    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            if self.has_stack().await {
                let stack = self.result.lock().await;
                stack.clone()
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }
}

/// `StonecutterScreenHandler` — vanilla `StonecutterMenu` equivalent.
///
/// Layout:
/// - Slot 0: Input slot (1 item)
/// - Slot 1: Output slot (result of selected recipe)
/// - Slots 2-28: Player inventory (3x9)
/// - Slots 29-37: Player hotbar (9)
///
/// Recipe selection:
/// When the input slot changes, `get_available_recipes()` returns all matching
/// stonecutting recipes from `RECIPES_STONECUTTING`. The client displays these
/// and sends the selected recipe index. `select_recipe()` sets the output slot.
pub struct StonecutterScreenHandler {
    pub input_inventory: Arc<StonecutterInventory>,
    pub output_slot: Arc<StonecutterOutputSlot>,
    /// Cached list of recipes available for the current input item.
    available_recipes: Vec<&'static StonecuttingRecipe>,
    /// Index of the currently selected recipe, or None if nothing selected.
    selected_recipe: Option<usize>,
    behaviour: ScreenHandlerBehaviour,
}

impl StonecutterScreenHandler {
    #[must_use]
    #[allow(clippy::unused_async)]
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let input_inventory = Arc::new(StonecutterInventory::new());
        let output_slot = Arc::new(StonecutterOutputSlot::new(input_inventory.clone()));

        let mut handler = Self {
            input_inventory: input_inventory.clone(),
            output_slot: output_slot.clone(),
            available_recipes: Vec::new(),
            selected_recipe: None,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Stonecutter)),
        };

        // Slot 0: Input
        handler.add_slot(Arc::new(NormalSlot::new(input_inventory, 0)));
        // Slot 1: Output
        handler.add_slot(output_slot);

        // Player inventory + hotbar
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    /// Returns all stonecutting recipes available for the current input item.
    pub fn get_available_recipes(&self) -> &[&'static StonecuttingRecipe] {
        &self.available_recipes
    }

    /// Updates the cached recipe list based on the current input item.
    /// Called when the input slot changes.
    pub async fn update_recipes(&mut self) {
        let input_stack = self.input_inventory.get_stack(0).await;
        let input = input_stack.lock().await;

        if input.is_empty() {
            self.available_recipes.clear();
            self.selected_recipe = None;
            self.output_slot.set_stack(ItemStack::EMPTY.clone()).await;
            return;
        }

        self.available_recipes = get_stonecutting_recipes_for(input.item);
        // Reset selection — client must re-select
        self.selected_recipe = None;
        self.output_slot.set_stack(ItemStack::EMPTY.clone()).await;
    }

    /// Selects a recipe by index from the available recipes list.
    /// Sets the output slot to the recipe result if valid.
    /// Returns true if the selection was valid.
    pub async fn select_recipe(&mut self, index: usize) -> bool {
        if index >= self.available_recipes.len() {
            return false;
        }

        // Verify input slot still has an item
        let input_stack = self.input_inventory.get_stack(0).await;
        let input = input_stack.lock().await;
        if input.is_empty() {
            return false;
        }

        let recipe = self.available_recipes[index];
        // Double-check ingredient still matches (input could have changed)
        if !recipe.ingredient.match_item(input.item) {
            self.selected_recipe = None;
            self.output_slot.set_stack(ItemStack::EMPTY.clone()).await;
            return false;
        }
        drop(input);

        self.selected_recipe = Some(index);
        let result = ItemStack::from(&recipe.result);
        self.output_slot.set_stack(result).await;
        true
    }
}

impl ScreenHandler for StonecutterScreenHandler {
    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.drop_inventory(player, self.input_inventory.clone())
                .await;
        })
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

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if !slot.has_stack().await {
                return ItemStack::EMPTY.clone();
            }

            let slot_stack = slot.get_stack().await;
            let mut slot_stack = slot_stack.lock().await;
            let stack_prev = slot_stack.clone();

            if slot_index == 1 {
                // From output slot — move to player inventory (2..38)
                if !self.insert_item(&mut slot_stack, 2, 38, true).await {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_take_item(player, &stack_prev).await;
            } else if slot_index == 0 {
                // From input slot — move to player inventory (2..38)
                if !self.insert_item(&mut slot_stack, 2, 38, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (2..38).contains(&slot_index) {
                // From player inventory — try input slot first (0..1)
                if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                    // Try within player inventory
                    if slot_index < 29 {
                        if !self.insert_item(&mut slot_stack, 29, 38, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 2, 29, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                }
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

            stack_prev
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stonecutter_inventory_size() {
        let inv = StonecutterInventory::new();
        assert_eq!(inv.size(), 1);
    }

    #[tokio::test]
    async fn stonecutter_inventory_starts_empty() {
        let inv = StonecutterInventory::new();
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_set_and_get() {
        let inv = StonecutterInventory::new();
        let item = ItemStack::new(5, &pumpkin_data::item::Item::STONE);
        inv.set_stack(0, item).await;

        let stack = inv.get_stack(0).await;
        assert_eq!(stack.lock().await.item_count, 5);
        assert!(!inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_clear() {
        let inv = StonecutterInventory::new();
        inv.set_stack(0, ItemStack::new(1, &pumpkin_data::item::Item::STONE))
            .await;
        assert!(!inv.is_empty().await);
        inv.clear().await;
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_remove_stack() {
        let inv = StonecutterInventory::new();
        inv.set_stack(0, ItemStack::new(3, &pumpkin_data::item::Item::STONE))
            .await;

        let removed = inv.remove_stack(0).await;
        assert_eq!(removed.item_count, 3);
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_cannot_insert() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);
        let item = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        assert!(!output.can_insert(&item).await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_starts_empty() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);
        assert!(!output.has_stack().await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_set_and_get() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);

        let item = ItemStack::new(2, &pumpkin_data::item::Item::STONE_SLAB);
        output.set_stack(item).await;

        assert!(output.has_stack().await);
        let stack = output.get_cloned_stack().await;
        assert_eq!(stack.item_count, 2);
    }

    // --- Recipe matching tests ---

    #[test]
    fn stonecutting_recipes_for_stone() {
        // Stone should have multiple stonecutting recipes (slabs, stairs, bricks, etc.)
        let recipes = get_stonecutting_recipes_for(&pumpkin_data::item::Item::STONE);
        assert!(
            !recipes.is_empty(),
            "Stone should have at least one stonecutting recipe"
        );

        // Verify all returned recipes actually match stone
        for recipe in &recipes {
            assert!(
                recipe.ingredient.match_item(&pumpkin_data::item::Item::STONE),
                "Recipe {} should match stone",
                recipe.recipe_id
            );
        }
    }

    #[test]
    fn stonecutting_recipes_for_dirt_is_empty() {
        // Dirt has no stonecutting recipes
        let recipes = get_stonecutting_recipes_for(&pumpkin_data::item::Item::DIRT);
        assert!(
            recipes.is_empty(),
            "Dirt should have no stonecutting recipes"
        );
    }

    #[test]
    fn stonecutting_recipes_for_andesite() {
        // Andesite should produce polished andesite, slabs, stairs, walls
        let recipes = get_stonecutting_recipes_for(&pumpkin_data::item::Item::ANDESITE);
        assert!(
            recipes.len() >= 2,
            "Andesite should have multiple stonecutting recipes, got {}",
            recipes.len()
        );
    }

    #[test]
    fn stonecutting_result_produces_valid_itemstack() {
        let recipes = get_stonecutting_recipes_for(&pumpkin_data::item::Item::STONE);
        assert!(!recipes.is_empty());
        let result = ItemStack::from(&recipes[0].result);
        assert!(!result.is_empty());
        assert!(result.item_count > 0);
    }

    #[test]
    fn stonecutting_total_recipe_count() {
        // Verify that RECIPES_STONECUTTING was actually generated with data
        assert_eq!(
            RECIPES_STONECUTTING.len(),
            254,
            "Expected 254 stonecutting recipes"
        );
    }
}
