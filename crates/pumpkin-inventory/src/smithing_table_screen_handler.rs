use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use crate::slot::{NormalSlot, Slot};

use pumpkin_data::data_component_impl::TrimImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{
    get_smithing_transform_recipe, get_smithing_trim_recipe, get_trim_material_for_item,
};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

pub struct SmithingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
}

impl SmithingTableScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Smithing));
        let input_inventory = Arc::new(SimpleInventory::new(3));
        let output_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            input_inventory,
            output_inventory,
        };

        handler.add_slot(Arc::new(NormalSlot::new(
            handler.input_inventory.clone(),
            0,
        )));
        handler.add_slot(Arc::new(NormalSlot::new(
            handler.input_inventory.clone(),
            1,
        )));
        handler.add_slot(Arc::new(NormalSlot::new(
            handler.input_inventory.clone(),
            2,
        )));
        handler.add_slot(Arc::new(SmithingOutputSlot::new(
            handler.output_inventory.clone(),
            handler.input_inventory.clone(),
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub fn update_output(&self) {
        let template_stack = self.input_inventory.get_stack(0);
        let base_stack = self.input_inventory.get_stack(1);
        let addition_stack = self.input_inventory.get_stack(2);

        if template_stack.is_empty() || base_stack.is_empty() || addition_stack.is_empty() {
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
            return;
        }

        if let Some(recipe) =
            get_smithing_transform_recipe(template_stack.item, base_stack.item, addition_stack.item)
        {
            let res_key = recipe
                .result
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(recipe.result.id);
            if let Some(target_item) = Item::from_registry_key(res_key) {
                let mut result = base_stack.clone();
                result.item = target_item;
                result.item_count = recipe.result.count;
                self.output_inventory.set_stack(0, result);
                return;
            }
        }

        if let Some(recipe) =
            get_smithing_trim_recipe(template_stack.item, base_stack.item, addition_stack.item)
            && let Some(material) = get_trim_material_for_item(addition_stack.item)
        {
            if let Some(trim) = base_stack.get_data_component::<TrimImpl>() {
                let curr_mat = match &trim.material {
                    NbtTag::String(s) => s.as_ref(),
                    _ => "",
                };
                let curr_pat = match &trim.pattern {
                    NbtTag::String(s) => s.as_ref(),
                    _ => "",
                };
                if curr_mat == material && curr_pat == recipe.pattern {
                    self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
                    return;
                }
            }

            let mut result = base_stack.clone();
            result.item_count = 1;
            result.set_data_component(TrimImpl {
                material: NbtTag::String(material.into()),
                pattern: NbtTag::String(recipe.pattern.into()),
            });

            self.output_inventory.set_stack(0, result);
            return;
        }

        self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
    }
}

impl ScreenHandler for SmithingTableScreenHandler {
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
            self.update_output();
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
                        // From output slot to player inventory
                        if !self.insert_item(&mut slot_stack, 4, 40, true) {
                            return ItemStack::EMPTY.clone();
                        }
                        slot.on_quick_move_crafted(slot_stack.clone(), stack.clone());
                        let mut taken_stack = stack.clone();
                        taken_stack.set_count(stack.item_count - slot_stack.item_count);
                        slot.on_take_item(player, &taken_stack);
                    }
                    std::cmp::Ordering::Less => {
                        // From input slots to player inventory
                        if !self.insert_item(&mut slot_stack, 4, 40, true) {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        // From player inventory into input slots (0..3)
                        if !self.insert_item(&mut slot_stack, 0, 3, false) {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                }
                self.update_output();

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

pub struct SmithingOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub input_inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl SmithingOutputSlot {
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

impl Slot for SmithingOutputSlot {
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
        self.input_inventory.remove_stack_specific(0, 1);
        self.input_inventory.remove_stack_specific(1, 1);
        self.input_inventory.remove_stack_specific(2, 1);
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

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {
        // Do nothing
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}
