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

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use std::any::Any;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use super::recipe_provider::{GenericRecipe, RecipeProvider};
use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
};
use crate::slot::{NormalSlot, Slot};

use crate::inventory::Inventory;
use pumpkin_data::data_component_impl::{PotDecorationsImpl, ProvidesPotteryPatternImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::codec::recipe::{DynamicRecipe, OwnedCraftingRecipe};

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
    pub pot_decorations: Option<[u16; 4]>,
    pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
}

impl RecipeResult {
    #[must_use]
    pub fn to_item_stack(&self) -> ItemStack {
        let key = self
            .item_id
            .strip_prefix("minecraft:")
            .unwrap_or(&self.item_id);
        let item = pumpkin_data::item::Item::from_registry_key(key)
            .unwrap_or(&pumpkin_data::item::Item::AIR);
        let mut stack = ItemStack::new(self.count, item);
        for (component, value) in self.components {
            stack.patch.push((*component, Some(value.clone_dyn())));
        }
        if let Some(sherds) = self.pot_decorations {
            let sherd_names = sherds
                .into_iter()
                .map(|id| {
                    let reg_key =
                        pumpkin_data::item::Item::from_id(id).map_or("brick", |it| it.registry_key);
                    if reg_key.starts_with("minecraft:") {
                        std::borrow::Cow::Borrowed(reg_key)
                    } else {
                        std::borrow::Cow::Owned(format!("minecraft:{reg_key}"))
                    }
                })
                .collect();
            stack.set_data_component(PotDecorationsImpl {
                sherds: sherd_names,
            });
        }
        stack
    }
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
fn recipe_matches(
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
            #[allow(clippy::redundant_closure_for_method_calls)]
            if pattern.len() != input_height
                || pattern.first().map_or(0, |f| f.len()) != input_width
            {
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
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset));
                    if current_key == ' ' {
                        if !slot.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }

                    let Some(ingredient) = key
                        .iter()
                        .find_map(|(k, v)| (*k == current_key).then_some(v))
                    else {
                        matched = false;
                        break 'outer;
                    };

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
                        let Some(current_key) = pattern[y].chars().nth(x) else {
                            matched = false;
                            break 'outer;
                        };
                        let slot = inventory.get_stack(
                            (y + y_offset) * inventory.get_height()
                                + (x_offset + input_width - 1 - x),
                        );
                        if current_key == ' ' {
                            if !slot.is_empty() {
                                matched = false;
                                break 'outer;
                            }
                            continue;
                        }
                        let Some(ingredient) = key
                            .iter()
                            .find_map(|(k, v)| (*k == current_key).then_some(v))
                        else {
                            matched = false;
                            break 'outer;
                        };
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
                pot_decorations: None,
                components: result.components,
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
                let slot = inventory.get_stack(i);
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
                pot_decorations: None,
                components: result.components,
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
                let slot = inventory.get_stack(i);
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
                pot_decorations: None,
                components: result.components,
            })
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingDecoratedPot { .. }) => {
            if count != 4 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            let mut sherds = [0u16; 4];
            for (idx, position) in (1..=7).step_by(2).enumerate() {
                let slot = inventory.get_stack(position);
                if slot.is_empty()
                    || (!slot
                        .item
                        .has_tag(&tag::Item::MINECRAFT_DECORATED_POT_INGREDIENTS)
                        && slot
                            .get_data_component::<ProvidesPotteryPatternImpl>()
                            .is_none())
                {
                    return None;
                }
                sherds[idx] = slot.item.id;
            }
            Some(RecipeResult {
                item_id: "minecraft:decorated_pot".to_string(),
                count: 1,
                pot_decorations: Some(sherds),
                components: &[],
            })
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shaped {
            pattern,
            key,
            result,
            ..
        }) => {
            #[allow(clippy::redundant_closure_for_method_calls)]
            if pattern.len() != input_height
                || pattern.first().map_or(0, |f| f.len()) != input_width
            {
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
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset));
                    if current_key == ' ' {
                        if !slot.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }
                    let Some(ingredient) =
                        key.iter().find(|(k, _)| *k == current_key).map(|(_, v)| v)
                    else {
                        matched = false;
                        break 'outer;
                    };
                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }
            matched.then_some(RecipeResult {
                item_id: result.item_id.clone(),
                count: result.count,
                pot_decorations: None,
                components: &[],
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
                let slot = inventory.get_stack(i);
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
                pot_decorations: None,
                components: &[],
            })
        }
        _ => None,
    }
}

#[must_use]
pub fn match_crafting_recipe(
    inventory: &dyn RecipeInputInventory,
    provider: Option<&dyn RecipeProvider>,
) -> Option<RecipeResult> {
    let mut count: usize = 0;
    let inventory_width = inventory.get_width();
    let mut top_x = 9;
    let mut top_y = 9;
    let mut bottom_x = 0;
    let mut bottom_y = 0;
    for i in 0..inventory.size() {
        let x = i % inventory_width;
        let y = i / inventory_width;
        let slot = inventory.get_stack(i);
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
            inventory,
        ) {
            return Some(result);
        }
    }

    if let Some(provider) = provider {
        let dynamic = provider.get_dynamic_recipes();
        for recipe in &dynamic {
            if let DynamicRecipe::Crafting(crafting) = recipe
                && let Some(result) = recipe_matches(
                    GenericRecipe::Dynamic(crafting),
                    input_height,
                    input_width,
                    top_x,
                    top_y,
                    count,
                    inventory,
                )
            {
                return Some(result);
            }
        }
    }

    None
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

    fn match_recipe(&self) -> Option<RecipeResult> {
        match_crafting_recipe(&*self.inventory, self.recipe_provider.as_deref())
    }

    fn refill_output(&self) -> ItemStack {
        let result = self.match_recipe().map_or_else(
            || ItemStack::EMPTY.clone(),
            |matched| matched.to_item_stack(),
        );
        *self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = result.clone();
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
    fn on_quick_move_crafted(&self, _stack: ItemStack, _stack_prev: ItemStack) {
        self.refill_output();
    }
    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        player.on_recipe_crafted(stack);
        for i in 0..self.inventory.size() {
            let ingredient = self.inventory.get_stack(i);
            if ingredient.is_empty() {
                continue;
            }
            let remainder = ingredient.crafting_remainder();
            self.inventory.remove_stack_specific(i, 1);
            let Some(remainder) = remainder else {
                continue;
            };

            let remaining = self.inventory.get_stack(i);
            if remaining.is_empty() {
                self.inventory.set_stack(i, remainder);
            } else if remaining.are_items_and_components_equal(&remainder) {
                let mut merged = remaining;
                merged.increment(remainder.item_count);
                self.inventory.set_stack(i, merged);
            } else {
                let mut leftover = remainder;
                if !player.get_inventory().insert_stack_anywhere(&mut leftover) {
                    player.drop_item(leftover, false);
                }
            }
        }
        self.mark_dirty();
    }
    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }
    fn get_stack(&self) -> ItemStack {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn get_cloned_stack(&self) -> ItemStack {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn has_stack(&self) -> bool {
        !self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }
    fn set_stack(&self, _stack: ItemStack) {
        self.refill_output();
    }
    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {
        self.refill_output();
    }
    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
    fn get_max_item_count(&self) -> u8 {
        let mut count = u8::MAX;
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i);
            if !slot.is_empty() {
                count = count.min(slot.item_count);
            }
        }
        count
    }
    fn take_stack(&self, _amount: u8) -> ItemStack {
        if self.has_stack() {
            self.result
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            ItemStack::EMPTY.clone()
        }
    }
}

impl ScreenHandlerListener for ResultSlot {
    fn on_slot_update(&self, screen_handler: &ScreenHandlerBehaviour, slot: u8, _stack: ItemStack) {
        if (0..=(self.inventory.get_width() * self.inventory.get_height()))
            .contains(&(slot as usize))
        {
            let result = self.refill_output();
            let next_revision = screen_handler.next_revision();
            if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                sync_handler.update_slot(screen_handler, 0, &result, next_revision);
            }
        }
    }
}

pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    fn add_recipe_slots(
        &mut self,
        crafing_inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) {
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
        self.add_listener(result_slot);
    }
}

pub struct CraftingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl CraftingTableScreenHandler {
    pub fn new(
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
        crafting_table_handler.add_recipe_slots(crafting_inventory, provider);
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
    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.drop_inventory(player, self.crafting_inventory.clone());
    }
    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();
        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            let stack_prev = slot_stack.clone();
            if slot_index == 0 {
                if !self.insert_item(&mut slot_stack, 10, 46, true) {
                    return ItemStack::EMPTY.clone();
                }
            } else if (1..=9).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 10, 46, false) {
                    return ItemStack::EMPTY.clone();
                }
            } else if (10..46).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 1, 10, false) {
                    if slot_index < 37 {
                        if !self.insert_item(&mut slot_stack, 37, 46, false) {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 10, 37, false) {
                        return ItemStack::EMPTY.clone();
                    }
                }
            } else if !self.insert_item(&mut slot_stack, 10, 46, false) {
                return ItemStack::EMPTY.clone();
            }
            let stack = slot_stack.clone();
            drop(slot_stack);
            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone());
            } else {
                slot.mark_dirty();
            }
            if stack.item_count == stack_prev.item_count {
                return ItemStack::EMPTY.clone();
            }

            let mut taken_stack = stack_prev.clone();
            taken_stack.set_count(stack_prev.item_count - stack.item_count);
            slot.on_take_item(player, &taken_stack);

            if slot_index == 0 {
                slot.on_quick_move_crafted(stack.clone(), stack_prev.clone());
                if !stack.is_empty() {
                    player.drop_item(stack, false);
                }
            }
            return stack_prev;
        }
        ItemStack::EMPTY.clone()
    }
}

impl CraftingScreenHandler<CraftingInventory> for CraftingTableScreenHandler {}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::data_component_impl::EquipmentSlot;
    use pumpkin_data::item::Item;
    use pumpkin_data::sound::Sound;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory, CSetSelectedSlot,
    };
    use pumpkin_protocol::java::server::play::SlotActionType;

    use crate::entity_equipment::EntityEquipment;
    use crate::screen_handler::InventoryPlayer;

    struct TestPlayer {
        inventory: Arc<PlayerInventory>,
        drops: Mutex<Vec<ItemStack>>,
    }

    impl TestPlayer {
        fn new(inventory: Arc<PlayerInventory>) -> Self {
            Self {
                inventory,
                drops: Mutex::new(Vec::new()),
            }
        }
    }

    impl InventoryPlayer for TestPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn drop_item(&self, item: ItemStack, _retain_ownership: bool) {
            self.drops
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(item);
        }

        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }

        fn has_infinite_materials(&self) -> bool {
            false
        }

        fn is_creative(&self) -> bool {
            false
        }

        fn experience_level(&self) -> i32 {
            0
        }

        fn add_experience_levels(&self, _levels: i32) {}

        fn enchantment_seed(&self) -> i32 {
            0
        }

        fn set_enchantment_seed(&self, _seed: i32) {}

        fn enqueue_inventory_packet(
            &self,
            _packet: &CSetContainerContent,
            _window_type: Option<WindowType>,
        ) {
        }

        fn enqueue_slot_packet(
            &self,
            _packet: &CSetContainerSlot,
            _window_type: Option<WindowType>,
            _total_slots: usize,
        ) {
        }

        fn enqueue_cursor_packet(&self, _packet: &CSetCursorItem) {}

        fn enqueue_property_packet(
            &self,
            _packet: &CSetContainerProperty,
            _window_type: Option<WindowType>,
        ) {
        }

        fn enqueue_slot_set_packet(&self, _packet: &CSetPlayerInventory) {}

        fn enqueue_set_held_item_packet(&self, _packet: &CSetSelectedSlot) {}

        fn enqueue_equipment_change(&self, _slot: &EquipmentSlot, _stack: &ItemStack) {}

        fn award_experience(&self, _amount: i32) {}

        fn increment_stat(&self, _category: StatisticCategory, _stat_id: i32, _amount: i32) {}

        fn play_block_sound(&self, _sound: Sound, _pitch: f32) {}
    }

    fn test_handler() -> (CraftingTableScreenHandler, Arc<PlayerInventory>, TestPlayer) {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(rustc_hash::FxHashMap::default()),
        ));
        let player = TestPlayer::new(player_inventory.clone());
        let handler = CraftingTableScreenHandler::new(1, &player_inventory, None);
        (handler, player_inventory, player)
    }

    fn set_slot(handler: &CraftingTableScreenHandler, slot: usize, stack: ItemStack) {
        handler.get_behaviour().slots[slot].set_stack(stack);
    }

    fn grid_stack(handler: &CraftingTableScreenHandler, slot: usize) -> ItemStack {
        handler.get_behaviour().slots[slot].get_stack()
    }

    fn refresh_result(handler: &CraftingTableScreenHandler) {
        handler.get_behaviour().slots[0].set_stack(ItemStack::EMPTY.clone());
    }

    fn place_cake(handler: &CraftingTableScreenHandler) {
        for slot in 1..=3 {
            set_slot(handler, slot, ItemStack::new(1, &Item::MILK_BUCKET));
        }
        set_slot(handler, 4, ItemStack::new(1, &Item::SUGAR));
        set_slot(handler, 5, ItemStack::new(1, &Item::EGG));
        set_slot(handler, 6, ItemStack::new(1, &Item::SUGAR));
        for slot in 7..=9 {
            set_slot(handler, slot, ItemStack::new(1, &Item::WHEAT));
        }
    }

    fn mace_result(grid_size: u8, top_slot: usize) -> Option<String> {
        let inventory = CraftingInventory::new(grid_size, grid_size);
        inventory.set_stack(top_slot, ItemStack::new(1, &Item::HEAVY_CORE));
        inventory.set_stack(
            top_slot + usize::from(grid_size),
            ItemStack::new(1, &Item::BREEZE_ROD),
        );
        match_crafting_recipe(&inventory, None).map(|result| result.item_id)
    }

    #[test]
    fn padded_shaped_recipe_matches_in_any_column() {
        for (grid_size, top_slot) in [(2, 0), (2, 1), (3, 0), (3, 1), (3, 5)] {
            assert_eq!(
                mace_result(grid_size, top_slot).as_deref(),
                Some("minecraft:mace"),
                "grid {grid_size}x{grid_size}, top slot {top_slot}"
            );
        }
    }

    #[test]
    fn taking_cake_returns_empty_buckets_and_consumes_other_ingredients() {
        let (mut handler, _, player) = test_handler();
        place_cake(&handler);
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::CAKE.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        for slot in 1..=3 {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::BUCKET.id);
        }
        for slot in 4..=9 {
            assert!(grid_stack(&handler, slot).is_empty());
        }
    }

    #[test]
    fn taking_honey_block_returns_glass_bottles() {
        let (mut handler, _, player) = test_handler();
        for slot in [1, 2, 4, 5] {
            set_slot(&handler, slot, ItemStack::new(1, &Item::HONEY_BOTTLE));
        }
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::HONEY_BLOCK.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        for slot in [1, 2, 4, 5] {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::GLASS_BOTTLE.id);
        }
    }

    #[test]
    fn taking_sugar_returns_the_honey_bottle_remainder() {
        let (mut handler, _, player) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::HONEY_BOTTLE));
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::SUGAR.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        assert_eq!(grid_stack(&handler, 1).item.id, Item::GLASS_BOTTLE.id);
    }

    #[test]
    fn taking_a_recipe_without_remainders_still_consumes_the_grid() {
        let (mut handler, _, player) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::OAK_LOG));
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::OAK_PLANKS.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        assert!(grid_stack(&handler, 1).is_empty());
    }

    #[test]
    fn occupied_input_slot_drops_the_remainder() {
        let (mut handler, player_inventory, player) = test_handler();
        place_cake(&handler);
        set_slot(&handler, 1, ItemStack::new(2, &Item::MILK_BUCKET));
        for slot in 0..player_inventory.size() {
            player_inventory.set_stack(slot, ItemStack::new(64, &Item::DIRT));
        }
        refresh_result(&handler);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        let milk = grid_stack(&handler, 1);
        assert_eq!(milk.item.id, Item::MILK_BUCKET.id);
        assert_eq!(milk.item_count, 1);
        let drops = player
            .drops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item.id, Item::BUCKET.id);
    }

    #[test]
    fn shift_clicking_the_result_returns_remainders() {
        let (mut handler, _, player) = test_handler();
        place_cake(&handler);
        refresh_result(&handler);

        handler.on_slot_click(0, 0, SlotActionType::QuickMove, &player);

        for slot in 1..=3 {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::BUCKET.id);
        }
    }
}
