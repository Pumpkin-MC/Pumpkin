use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion_brewing::{ITEM_RECIPES, POTION_RECIPES};
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_world::block::entities::PropertyDelegate;
use pumpkin_world::inventory::Inventory;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty,
};
use crate::slot::Slot;

#[must_use]
pub fn is_fuel(item: &Item) -> bool {
    item.id == Item::BLAZE_POWDER.id || item.has_tag(&tag::Item::MINECRAFT_BREWING_FUEL)
}

#[must_use]
pub fn is_ingredient(item: &Item) -> bool {
    ITEM_RECIPES
        .iter()
        .any(|r| r.ingredient().iter().any(|i| i.id == item.id))
        || POTION_RECIPES
            .iter()
            .any(|r| r.ingredient().iter().any(|i| i.id == item.id))
}

#[must_use]
pub const fn is_potion_item(item: &Item) -> bool {
    item.id == Item::POTION.id
        || item.id == Item::SPLASH_POTION.id
        || item.id == Item::LINGERING_POTION.id
        || item.id == Item::GLASS_BOTTLE.id
}

pub struct BrewingScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    pub behaviour: ScreenHandlerBehaviour,
    pub property_delegate: Arc<dyn PropertyDelegate>,
}

impl BrewingScreenHandler {
    pub const BOTTLE_SLOT_START: usize = 0;
    pub const BOTTLE_SLOT_END: usize = 2;
    pub const INGREDIENT_SLOT: usize = 3;
    pub const FUEL_SLOT: usize = 4;
    pub const SLOT_COUNT: usize = 5;
    pub const DATA_COUNT: usize = 2;
    pub const INV_SLOT_START: usize = 5;
    pub const INV_SLOT_END: usize = 32;
    pub const USE_ROW_SLOT_START: usize = 32;
    pub const USE_ROW_SLOT_END: usize = 41;

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        property_delegate: &Arc<dyn PropertyDelegate>,
    ) -> Self {
        struct BrewingScreenListener;
        impl crate::screen_handler::ScreenHandlerListener for BrewingScreenListener {
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
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::BrewingStand)),
            property_delegate: property_delegate.clone(),
        };

        handler.add_property(ScreenProperty::new(property_delegate.clone(), 0));
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 1));

        handler.add_listener(Arc::new(BrewingScreenListener));

        for i in 0..3 {
            handler.add_slot(Arc::new(BrewingPotionSlot::new(inventory.clone(), i)));
        }
        handler.add_slot(Arc::new(BrewingIngredientSlot::new(inventory.clone(), 3)));
        handler.add_slot(Arc::new(BrewingFuelSlot::new(inventory, 4)));

        let pi: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&pi);

        handler
    }

    #[must_use]
    pub fn get_fuel(&self) -> i32 {
        self.property_delegate.get_property(1)
    }

    #[must_use]
    pub fn get_brewing_ticks(&self) -> i32 {
        self.property_delegate.get_property(0)
    }
}

impl ScreenHandler for BrewingScreenHandler {
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

    fn quick_move(&mut self, _player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut clicked = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot {
            if !slot.has_stack() {
                return clicked;
            }

            let mut stack = slot.get_stack();
            clicked = stack.clone();

            if slot_index >= 5 {
                if is_fuel(clicked.item) {
                    if self.insert_item(&mut stack, 4, 5, false)
                        || (is_ingredient(clicked.item)
                            && !self.insert_item(&mut stack, 3, 4, false))
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_ingredient(clicked.item) {
                    if !self.insert_item(&mut stack, 3, 4, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_potion_item(clicked.item) {
                    if !self.insert_item(&mut stack, 0, 3, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (5..32).contains(&slot_index) {
                    if !self.insert_item(&mut stack, 32, 41, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (32..41).contains(&slot_index) {
                    if !self.insert_item(&mut stack, 5, 32, false) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self.insert_item(&mut stack, 5, 41, false) {
                    return ItemStack::EMPTY.clone();
                }
            } else {
                if !self.insert_item(&mut stack, 5, 41, true) {
                    return ItemStack::EMPTY.clone();
                }

                slot.on_quick_move_crafted(stack.clone(), clicked.clone());
            }

            if stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(stack.clone());
            }

            if stack.item_count == clicked.item_count {
                return ItemStack::EMPTY.clone();
            }
        }

        clicked
    }
}

pub struct BrewingPotionSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
}

impl BrewingPotionSlot {
    #[must_use]
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for BrewingPotionSlot {
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
        is_potion_item(stack.item)
    }

    fn get_max_item_count(&self) -> u8 {
        1
    }

    fn get_max_item_count_for_stack(&self, _stack: &ItemStack) -> u8 {
        1
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

pub struct BrewingIngredientSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
}

impl BrewingIngredientSlot {
    #[must_use]
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for BrewingIngredientSlot {
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
        is_ingredient(stack.item)
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

pub struct BrewingFuelSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
}

impl BrewingFuelSlot {
    #[must_use]
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for BrewingFuelSlot {
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
        is_fuel(stack.item)
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

pub fn create_brewing(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
    property_delegate: &Arc<dyn PropertyDelegate>,
) -> Option<BrewingScreenHandler> {
    let handler =
        BrewingScreenHandler::new(sync_id, player_inventory, inventory, property_delegate);
    Some(handler)
}
