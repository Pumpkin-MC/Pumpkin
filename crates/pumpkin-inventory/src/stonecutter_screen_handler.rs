use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU8, AtomicU16, Ordering};

use crate::crafting::recipe_provider::RecipeProvider;
use crate::inventory::Inventory;
use crate::inventory::SimpleInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty,
};
use crate::slot::{NormalSlot, Slot};
use crate::window_property::PropertyDelegate;

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::RECIPES_STONECUTTING;
use pumpkin_data::screen::WindowType;
use pumpkin_data::sound::Sound;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_protocol::java::server::play::SlotActionType;

pub struct StonecutterRecipeChoice {
    pub result_id: String,
    pub count: u8,
    pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
}

impl StonecutterRecipeChoice {
    #[must_use]
    pub fn assemble(&self) -> ItemStack {
        let key = self
            .result_id
            .strip_prefix("minecraft:")
            .unwrap_or(&self.result_id);
        let Some(item) = Item::from_registry_key(key) else {
            return ItemStack::EMPTY.clone();
        };
        let mut stack = ItemStack::new(self.count, item);
        for (component, value) in self.components {
            stack.patch.push((*component, Some(value.clone_dyn())));
        }
        stack
    }
}

#[must_use]
pub fn get_stonecutter_recipes(
    input: &ItemStack,
    provider: Option<&dyn RecipeProvider>,
) -> Vec<StonecutterRecipeChoice> {
    if input.is_empty() {
        return Vec::new();
    }
    let item = input.item;
    let mut results = Vec::new();
    for r in RECIPES_STONECUTTING {
        if r.ingredient.match_item(item) {
            results.push(StonecutterRecipeChoice {
                result_id: r.result.id.to_string(),
                count: r.result.count,
                components: r.result.components,
            });
        }
    }
    if let Some(provider) = provider {
        for dynamic in provider.get_dynamic_recipes() {
            if let DynamicRecipe::Stonecutting(r) = dynamic
                && r.ingredient.match_item(item)
            {
                results.push(StonecutterRecipeChoice {
                    result_id: r.result.item_id,
                    count: r.result.count,
                    components: &[],
                });
            }
        }
    }
    results
}

pub struct StonecutterState {
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub selected_recipe_index: AtomicI32,
    pub input_item: AtomicU16,
    pub last_sound_time: AtomicI64,
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
}

impl StonecutterState {
    fn recipes_for_input(&self) -> Vec<StonecutterRecipeChoice> {
        get_stonecutter_recipes(
            &self.input_inventory.get_stack(0),
            self.recipe_provider.as_deref(),
        )
    }

    fn is_valid_recipe_index(&self, index: i32) -> bool {
        index >= 0 && (index as usize) < self.recipes_for_input().len()
    }

    pub fn slots_changed(&self) {
        let input = self.input_inventory.get_stack(0);
        let input_item = if input.is_empty() { 0 } else { input.item.id };
        if self.input_item.swap(input_item, Ordering::Relaxed) != input_item {
            self.selected_recipe_index.store(-1, Ordering::Relaxed);
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
        }
    }

    pub fn setup_result_slot(&self, index: i32) {
        let recipes = self.recipes_for_input();
        let result = if index >= 0
            && let Some(recipe) = recipes.get(index as usize)
        {
            recipe.assemble()
        } else {
            ItemStack::EMPTY.clone()
        };
        self.output_inventory.set_stack(0, result);
    }
}

impl PropertyDelegate for StonecutterState {
    fn get_property(&self, index: i32) -> i32 {
        if index == 0 {
            self.selected_recipe_index.load(Ordering::Relaxed)
        } else {
            0
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        if index == 0 {
            self.selected_recipe_index.store(value, Ordering::Relaxed);
        }
    }

    fn get_properties_size(&self) -> i32 {
        1
    }
}

pub struct StonecutterScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub state: Arc<StonecutterState>,
}

impl StonecutterScreenHandler {
    pub const INPUT_SLOT: i32 = 0;
    pub const RESULT_SLOT: i32 = 1;
    pub const INV_SLOT_START: i32 = 2;
    pub const INV_SLOT_END: i32 = 29;
    pub const USE_ROW_SLOT_START: i32 = 29;
    pub const USE_ROW_SLOT_END: i32 = 38;

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        recipe_provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Stonecutter));
        let input_inventory = Arc::new(SimpleInventory::new(1));
        let output_inventory = Arc::new(SimpleInventory::new(1));
        let state = Arc::new(StonecutterState {
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
            selected_recipe_index: AtomicI32::new(-1),
            input_item: AtomicU16::new(0),
            last_sound_time: AtomicI64::new(0),
            recipe_provider,
        });

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
            state: state.clone(),
        };

        handler.add_slot(Arc::new(NormalSlot::new(
            input_inventory as Arc<dyn Inventory>,
            0,
        )));
        handler.add_slot(Arc::new(StonecutterOutputSlot::new(
            output_inventory as Arc<dyn Inventory>,
            state.clone(),
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);
        handler.add_property(ScreenProperty::new(state, 0));

        handler
    }

    pub fn update_output(&self) {
        self.state.slots_changed();
        self.state
            .setup_result_slot(self.state.selected_recipe_index.load(Ordering::Relaxed));
    }

    pub fn set_selected_recipe(&self, index: u8) {
        self.state.slots_changed();
        let index = if index == u8::MAX {
            -1
        } else {
            i32::from(index)
        };
        if self.state.is_valid_recipe_index(index) {
            self.state
                .selected_recipe_index
                .store(index, Ordering::Relaxed);
            self.state.setup_result_slot(index);
        } else {
            self.state
                .selected_recipe_index
                .store(-1, Ordering::Relaxed);
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
        }
    }

    #[must_use]
    pub fn get_available_recipes(&self, input: &ItemStack) -> Vec<StonecutterRecipeChoice> {
        get_stonecutter_recipes(input, self.state.recipe_provider.as_deref())
    }

    fn accepts_input(&self, stack: &ItemStack) -> bool {
        !self.get_available_recipes(stack).is_empty()
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

    fn on_button_click(&mut self, _player: &dyn InventoryPlayer, button_id: i32) -> bool {
        if self.state.selected_recipe_index.load(Ordering::Relaxed) == button_id {
            return false;
        }
        if self.state.is_valid_recipe_index(button_id) {
            self.state
                .selected_recipe_index
                .store(button_id, Ordering::Relaxed);
            self.state.setup_result_slot(button_id);
        }
        true
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        self.state.slots_changed();
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let Some(slot) = self.get_behaviour().slots.get(slot_index as usize).cloned() else {
            return ItemStack::EMPTY.clone();
        };
        if !slot.has_stack() {
            return ItemStack::EMPTY.clone();
        }

        let mut stack = slot.get_cloned_stack();
        let clicked = stack.clone();
        if slot_index == Self::RESULT_SLOT {
            player.on_crafted_post_process(&mut stack);
            if !self.insert_item(
                &mut stack,
                Self::INV_SLOT_START,
                Self::USE_ROW_SLOT_END,
                true,
            ) {
                return ItemStack::EMPTY.clone();
            }
            slot.on_quick_move_crafted(stack.clone(), clicked.clone());
        } else if slot_index == Self::INPUT_SLOT {
            if !self.insert_item(
                &mut stack,
                Self::INV_SLOT_START,
                Self::USE_ROW_SLOT_END,
                false,
            ) {
                return ItemStack::EMPTY.clone();
            }
        } else if self.accepts_input(&stack) {
            if !self.insert_item(&mut stack, Self::INPUT_SLOT, Self::RESULT_SLOT, false) {
                return ItemStack::EMPTY.clone();
            }
        } else if (Self::INV_SLOT_START..Self::INV_SLOT_END).contains(&slot_index) {
            if !self.insert_item(
                &mut stack,
                Self::USE_ROW_SLOT_START,
                Self::USE_ROW_SLOT_END,
                false,
            ) {
                return ItemStack::EMPTY.clone();
            }
        } else if (Self::USE_ROW_SLOT_START..Self::USE_ROW_SLOT_END).contains(&slot_index)
            && !self.insert_item(&mut stack, Self::INV_SLOT_START, Self::INV_SLOT_END, false)
        {
            return ItemStack::EMPTY.clone();
        }

        slot.set_stack(if stack.is_empty() {
            ItemStack::EMPTY.clone()
        } else {
            stack.clone()
        });
        slot.mark_dirty();
        self.state.slots_changed();

        if stack.item_count == clicked.item_count {
            return ItemStack::EMPTY.clone();
        }

        slot.on_take_item(player, &stack);
        if slot_index == Self::RESULT_SLOT && !stack.is_empty() {
            player.drop_item(stack, false);
        }

        clicked
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
        self.drop_inventory(player, self.input_inventory.clone());
    }
}

pub struct StonecutterOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub state: Arc<StonecutterState>,
    pub index: usize,
    pub id: AtomicU8,
}

impl StonecutterOutputSlot {
    pub fn new(inventory: Arc<dyn Inventory>, state: Arc<StonecutterState>, index: usize) -> Self {
        Self {
            inventory,
            state,
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

    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        player.on_recipe_crafted(stack);
        let remaining = self.state.input_inventory.remove_stack_specific(0, 1);
        if !remaining.is_empty() {
            self.state
                .setup_result_slot(self.state.selected_recipe_index.load(Ordering::Relaxed));
        }
        self.state.slots_changed();

        let game_time = player.game_time();
        if self
            .state
            .last_sound_time
            .swap(game_time, Ordering::Relaxed)
            != game_time
        {
            player.play_block_sound(Sound::UiStonecutterTakeResult, 1.0);
        }
        self.mark_dirty();
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn get_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn get_cloned_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn has_stack(&self) -> bool {
        !self.inventory.get_stack(self.index).is_empty()
    }

    fn set_stack(&self, stack: ItemStack) {
        self.inventory.set_stack(self.index, stack);
    }

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {}

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}
