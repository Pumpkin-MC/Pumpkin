use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty,
};
use crate::slot::Slot;

use pumpkin_data::data_component_impl::{BannerPatternLayer, BannerPatternsImpl};
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag::{BannerPattern, RegistryKey, get_tag_values};
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::block::entities::PropertyDelegate;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

pub const NO_ITEM_REQUIRED_PATTERNS: &[&str] = BannerPattern::MINECRAFT_NO_ITEM_REQUIRED.0;

#[must_use]
pub fn is_banner_item(item: &Item) -> bool {
    item.registry_key.ends_with("_banner")
}

#[must_use]
pub fn is_dye_item(item: &Item) -> bool {
    item.registry_key.ends_with("_dye")
}

#[must_use]
pub fn is_pattern_item(item: &Item) -> bool {
    item.registry_key.ends_with("_banner_pattern")
}

#[must_use]
pub fn get_dye_color(stack: &ItemStack) -> Option<DyeColor> {
    stack
        .item
        .registry_key
        .strip_suffix("_dye")
        .and_then(DyeColor::by_name)
}

#[must_use]
pub fn get_selectable_patterns(pattern_stack: &ItemStack) -> Vec<String> {
    if pattern_stack.is_empty() {
        return BannerPattern::MINECRAFT_NO_ITEM_REQUIRED
            .0
            .iter()
            .map(|s| {
                if s.contains(':') {
                    (*s).to_string()
                } else {
                    format!("minecraft:{s}")
                }
            })
            .collect();
    }

    let item_key = pattern_stack
        .item
        .registry_key
        .strip_suffix("_banner_pattern")
        .unwrap_or(pattern_stack.item.registry_key);
    let tag_name = format!("minecraft:pattern_item/{item_key}");
    get_tag_values(RegistryKey::BannerPattern, &tag_name).map_or_else(Vec::new, |values| {
        values
            .iter()
            .map(|s| {
                if s.contains(':') {
                    (*s).to_string()
                } else {
                    format!("minecraft:{s}")
                }
            })
            .collect()
    })
}

pub struct LoomPropertyDelegate(Arc<AtomicI32>);

impl PropertyDelegate for LoomPropertyDelegate {
    fn get_property(&self, index: i32) -> i32 {
        if index == 0 {
            self.0.load(Ordering::Relaxed)
        } else {
            0
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        if index == 0 {
            self.0.store(value, Ordering::Relaxed);
        }
    }

    fn get_properties_size(&self) -> i32 {
        1
    }
}

pub struct LoomScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub selected_banner_pattern_index: Arc<AtomicI32>,
    pub selectable_patterns: Vec<String>,
}

impl LoomScreenHandler {
    pub const PATTERN_NOT_SET: i32 = -1;

    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Loom));
        let input_inventory = Arc::new(SimpleInventory::new(3));
        let output_inventory = Arc::new(SimpleInventory::new(1));
        let selected_banner_pattern_index = Arc::new(AtomicI32::new(Self::PATTERN_NOT_SET));

        let property_delegate =
            Arc::new(LoomPropertyDelegate(selected_banner_pattern_index.clone()));

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
            selected_banner_pattern_index: selected_banner_pattern_index.clone(),
            selectable_patterns: Vec::new(),
        };

        handler.add_property(ScreenProperty::new(property_delegate, 0));

        handler.add_slot(Arc::new(LoomSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            0,
            LoomSlotType::Banner,
        )));
        handler.add_slot(Arc::new(LoomSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            1,
            LoomSlotType::Dye,
        )));
        handler.add_slot(Arc::new(LoomSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            2,
            LoomSlotType::Pattern,
        )));
        handler.add_slot(Arc::new(LoomResultSlot::new(
            output_inventory as Arc<dyn Inventory>,
            input_inventory as Arc<dyn Inventory>,
            selected_banner_pattern_index,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub fn slots_changed(&mut self) {
        let banner_stack = self.input_inventory.get_stack(0);
        let dye_stack = self.input_inventory.get_stack(1);
        let pattern_stack = self.input_inventory.get_stack(2);

        if !banner_stack.is_empty() && !dye_stack.is_empty() {
            let selected_pattern = self.selected_banner_pattern_index.load(Ordering::Relaxed);
            let valid_pattern_index = selected_pattern >= 0
                && (selected_pattern as usize) < self.selectable_patterns.len();
            let previous_selectable_patterns = self.selectable_patterns.clone();
            self.selectable_patterns = get_selectable_patterns(&pattern_stack);

            let pattern_to_display = if self.selectable_patterns.len() == 1 {
                self.selected_banner_pattern_index
                    .store(0, Ordering::Relaxed);
                self.selectable_patterns.first().cloned()
            } else if !valid_pattern_index {
                self.selected_banner_pattern_index
                    .store(Self::PATTERN_NOT_SET, Ordering::Relaxed);
                None
            } else {
                let selected_value = &previous_selectable_patterns[selected_pattern as usize];
                if let Some(new_selected_index) = self
                    .selectable_patterns
                    .iter()
                    .position(|p| p == selected_value)
                {
                    self.selected_banner_pattern_index
                        .store(new_selected_index as i32, Ordering::Relaxed);
                    Some(selected_value.clone())
                } else {
                    self.selected_banner_pattern_index
                        .store(Self::PATTERN_NOT_SET, Ordering::Relaxed);
                    None
                }
            };

            if let Some(pattern) = pattern_to_display {
                let layers_count = banner_stack
                    .get_data_component::<BannerPatternsImpl>()
                    .map_or(0, |bp| bp.layers.len());
                let has_max_patterns = layers_count >= 6;
                if has_max_patterns {
                    self.selected_banner_pattern_index
                        .store(Self::PATTERN_NOT_SET, Ordering::Relaxed);
                    self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
                } else {
                    self.setup_result_slot(&pattern);
                }
            } else {
                self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
            }
        } else {
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
            self.selectable_patterns = Vec::new();
            self.selected_banner_pattern_index
                .store(Self::PATTERN_NOT_SET, Ordering::Relaxed);
        }
    }

    fn setup_result_slot(&self, pattern: &str) {
        let mut banner_stack = self.input_inventory.get_stack(0);
        let dye_stack = self.input_inventory.get_stack(1);

        let result = if !banner_stack.is_empty()
            && !dye_stack.is_empty()
            && let Some(pattern_color) = get_dye_color(&dye_stack)
        {
            banner_stack.item_count = 1;

            let mut existing_layers = banner_stack
                .get_data_component::<BannerPatternsImpl>()
                .cloned()
                .unwrap_or_default();
            existing_layers.layers.push(BannerPatternLayer {
                pattern: pattern.to_string(),
                color: pattern_color,
            });
            banner_stack.set_data_component(existing_layers);
            banner_stack
        } else {
            ItemStack::EMPTY.clone()
        };

        let current_result = self.output_inventory.get_stack(0);
        if !current_result.are_items_and_components_equal(&result) {
            self.output_inventory.set_stack(0, result);
        }
    }
}

impl ScreenHandler for LoomScreenHandler {
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

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        if (0..=3).contains(&slot_index) {
            self.slots_changed();
        }
    }

    fn on_button_click(&mut self, _player: &dyn InventoryPlayer, button_id: i32) -> bool {
        if button_id >= 0 && (button_id as usize) < self.selectable_patterns.len() {
            self.selected_banner_pattern_index
                .store(button_id, Ordering::Relaxed);
            let pattern = self.selectable_patterns[button_id as usize].clone();
            self.setup_result_slot(&pattern);
            true
        } else {
            false
        }
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot {
            let mut slot_stack = slot.get_cloned_stack();
            if !slot_stack.is_empty() {
                stack = slot_stack.clone();
                match slot_index.cmp(&3) {
                    std::cmp::Ordering::Equal => {
                        if !self.insert_item(&mut slot_stack, 4, 40, true) {
                            return ItemStack::EMPTY.clone();
                        }
                        slot.on_quick_move_crafted(slot_stack.clone(), stack.clone());
                        let mut taken_stack = stack.clone();
                        taken_stack.set_count(stack.item_count - slot_stack.item_count);
                        slot.on_take_item(player, &taken_stack);
                    }
                    std::cmp::Ordering::Less => {
                        if !self.insert_item(&mut slot_stack, 4, 40, false) {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        if is_banner_item(slot_stack.item) {
                            if !self.insert_item(&mut slot_stack, 0, 1, false) {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if is_dye_item(slot_stack.item) {
                            if !self.insert_item(&mut slot_stack, 1, 2, false) {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if is_pattern_item(slot_stack.item) {
                            if !self.insert_item(&mut slot_stack, 2, 3, false) {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if (4..31).contains(&slot_index) {
                            if !self.insert_item(&mut slot_stack, 31, 40, false) {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if (31..40).contains(&slot_index)
                            && !self.insert_item(&mut slot_stack, 4, 31, false)
                        {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                }
                self.slots_changed();

                if slot_stack.is_empty() {
                    slot.set_stack(ItemStack::EMPTY.clone());
                } else {
                    slot.set_stack(slot_stack);
                }
            }
        }
        stack
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.drop_inventory(player, self.input_inventory.clone());
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoomSlotType {
    Banner,
    Dye,
    Pattern,
}

pub struct LoomSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
    slot_type: LoomSlotType,
}

impl LoomSlot {
    pub fn new(inventory: Arc<dyn Inventory>, index: usize, slot_type: LoomSlotType) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            slot_type,
        }
    }
}

impl Slot for LoomSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> bool {
        match self.slot_type {
            LoomSlotType::Banner => is_banner_item(stack.item),
            LoomSlotType::Dye => is_dye_item(stack.item),
            LoomSlotType::Pattern => is_pattern_item(stack.item),
        }
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

pub struct LoomResultSlot {
    inventory: Arc<dyn Inventory>,
    input_inventory: Arc<dyn Inventory>,
    selected_pattern_index: Arc<AtomicI32>,
    index: usize,
    id: AtomicU8,
}

impl LoomResultSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        input_inventory: Arc<dyn Inventory>,
        selected_pattern_index: Arc<AtomicI32>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            input_inventory,
            selected_pattern_index,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for LoomResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        self.input_inventory.remove_stack_specific(0, 1);
        self.input_inventory.remove_stack_specific(1, 1);
        if self.input_inventory.get_stack(0).is_empty()
            || self.input_inventory.get_stack(1).is_empty()
        {
            self.selected_pattern_index
                .store(LoomScreenHandler::PATTERN_NOT_SET, Ordering::Relaxed);
        }
        self.mark_dirty();
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
