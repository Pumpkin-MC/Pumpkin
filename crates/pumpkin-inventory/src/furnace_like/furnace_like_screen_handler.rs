use std::any::Any;
use std::sync::Arc;

use pumpkin_data::fuels::is_fuel;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CookingRecipeKind, CookingRecipeType};
use pumpkin_data::screen::WindowType;
use pumpkin_world::{
    block::entities::{ExperienceContainer, PropertyDelegate},
    inventory::Inventory,
};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
        ScreenProperty,
    },
};

use super::furnace_like_slot::{FurnaceLikeSlot, FurnaceLikeSlotType, FurnaceOutputSlot};

#[must_use]
pub fn can_smelt_item(item: &pumpkin_data::item::Item, window_type: WindowType) -> bool {
    let kind = match window_type {
        WindowType::Furnace => CookingRecipeKind::Smelting,
        WindowType::BlastFurnace => CookingRecipeKind::Blasting,
        WindowType::Smoker => CookingRecipeKind::Smoking,
        _ => return false,
    };

    pumpkin_data::recipes::RECIPES_COOKING
        .iter()
        .any(|recipe| match (recipe, &kind) {
            (CookingRecipeType::Smelting(r), CookingRecipeKind::Smelting)
            | (CookingRecipeType::Blasting(r), CookingRecipeKind::Blasting)
            | (CookingRecipeType::Smoking(r), CookingRecipeKind::Smoking) => {
                r.ingredient.match_item(item)
            }
            _ => false,
        })
}

pub struct FurnaceLikeScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    pub property_delegate: Arc<dyn PropertyDelegate>,
    pub experience_container: Arc<dyn ExperienceContainer>,
    pub window_type: WindowType,
    pub behaviour: ScreenHandlerBehaviour,
}

impl FurnaceLikeScreenHandler {
    pub const INGREDIENT_SLOT: usize = 0;
    pub const FUEL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const SLOT_COUNT: usize = 3;
    pub const DATA_COUNT: usize = 4;
    pub const INV_SLOT_START: usize = 3;
    pub const INV_SLOT_END: usize = 30;
    pub const USE_ROW_SLOT_START: usize = 30;
    pub const USE_ROW_SLOT_END: usize = 39;

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        property_delegate: &Arc<dyn PropertyDelegate>,
        experience_container: Arc<dyn ExperienceContainer>,
        window_type: WindowType,
    ) -> Self {
        struct FurnaceLikeScreenListener;
        impl ScreenHandlerListener for FurnaceLikeScreenListener {
            fn on_property_update(
                &self,
                screen_handler: &ScreenHandlerBehaviour,
                property: u8,
                value: i32,
            ) {
                if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                    sync_handler.update_property(screen_handler, i32::from(property), value);
                }
            }
        }

        let mut handler = Self {
            inventory,
            property_delegate: property_delegate.clone(),
            experience_container,
            window_type,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(window_type)),
        };

        for i in 0..4 {
            handler.add_property(ScreenProperty::new(property_delegate.clone(), i));
        }

        handler.add_listener(Arc::new(FurnaceLikeScreenListener));
        handler.add_inventory_slots();
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    fn add_inventory_slots(&mut self) {
        self.add_slot(Arc::new(FurnaceLikeSlot::new(
            self.inventory.clone(),
            FurnaceLikeSlotType::Top,
        )));
        self.add_slot(Arc::new(FurnaceLikeSlot::new(
            self.inventory.clone(),
            FurnaceLikeSlotType::Bottom,
        )));
        self.add_slot(Arc::new(FurnaceOutputSlot::new(
            self.inventory.clone(),
            self.experience_container.clone(),
        )));
    }

    #[must_use]
    pub fn can_smelt(&self, item_stack: &ItemStack) -> bool {
        can_smelt_item(item_stack.item, self.window_type)
    }

    #[must_use]
    pub const fn is_fuel(&self, item_stack: &ItemStack) -> bool {
        is_fuel(item_stack.item.id)
    }

    #[must_use]
    pub fn get_burn_progress(&self) -> f32 {
        let current = self.property_delegate.get_property(2);
        let total = self.property_delegate.get_property(3);
        if total != 0 && current != 0 {
            ((current as f32) / (total as f32)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn get_lit_progress(&self) -> f32 {
        let mut lit_duration = self.property_delegate.get_property(1);
        if lit_duration == 0 {
            lit_duration = 200;
        }
        ((self.property_delegate.get_property(0) as f32) / (lit_duration as f32)).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn is_lit(&self) -> bool {
        self.property_delegate.get_property(0) > 0
    }
}

impl ScreenHandler for FurnaceLikeScreenHandler {
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
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut clicked = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot {
            if !slot.has_stack() {
                return clicked;
            }

            let mut stack = slot.get_stack();
            clicked = stack.clone();

            if slot_index == 2 {
                if !self.insert_item(&mut stack, 3, 39, true) {
                    return ItemStack::EMPTY.clone();
                }

                slot.on_quick_move_crafted(stack.clone(), clicked.clone());
            } else if slot_index != 1 && slot_index != 0 {
                if self.can_smelt(&stack) {
                    if !self.insert_item(&mut stack, 0, 1, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if self.is_fuel(&stack) {
                    if !self.insert_item(&mut stack, 1, 2, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (3..30).contains(&slot_index) {
                    if !self.insert_item(&mut stack, 30, 39, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (30..39).contains(&slot_index)
                    && !self.insert_item(&mut stack, 3, 30, false)
                {
                    return ItemStack::EMPTY.clone();
                }
            } else if !self.insert_item(&mut stack, 3, 39, false) {
                return ItemStack::EMPTY.clone();
            }

            if stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(stack.clone());
            }

            if stack.item_count == clicked.item_count {
                return ItemStack::EMPTY.clone();
            }

            let mut taken_stack = clicked.clone();
            taken_stack.set_count(clicked.item_count - stack.item_count);
            slot.on_take_item(player, &taken_stack);
        }

        clicked
    }
}

pub type FurnaceScreenHandler = FurnaceLikeScreenHandler;
pub type SmokerScreenHandler = FurnaceLikeScreenHandler;
pub type BlastFurnaceScreenHandler = FurnaceLikeScreenHandler;
