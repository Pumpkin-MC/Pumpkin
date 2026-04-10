//! Brewing stand screen handler.
//!
//! This module implements the screen handler for brewing stands.
//! Brewing stands have 5 slots:
//! - Slots 0-2: Potion bottles (output/input)
//! - Slot 3: Brewing ingredient
//! - Slot 4: Fuel (blaze powder)
//!
//! The brewing stand has two tracked properties:
//! - Brew time (0-400): Progress of the current brewing operation
//! - Fuel time (0-20): Amount of fuel remaining

use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::tag::Taggable;
use pumpkin_data::{screen::WindowType, tag};
use pumpkin_world::{block::entities::PropertyDelegate, inventory::Inventory};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture, ScreenProperty},
};

use pumpkin_data::item_stack::ItemStack;

/// Screen handler for the brewing stand.
///
/// Manages the brewing stand's 5 slots and tracks brewing progress
/// and fuel levels.
pub struct BrewingScreenHandler {
    /// The brewing stand's inventory (5 slots: 0-2 potions, 3 ingredient, 4 fuel).
    inventory: Arc<dyn Inventory>,
    /// Core screen handler behavior (slots, sync ID, listeners).
    behaviour: ScreenHandlerBehaviour,
    /// Delegate for accessing brew time and fuel time properties.
    ///
    /// Property 0: Brew time (0-400), Property 1: Fuel time (0-20).
    _property_delegate: Arc<dyn PropertyDelegate>,
}

impl BrewingScreenHandler {
    /// Creates a new brewing stand screen handler.
    ///
    /// # Arguments
    /// - `sync_id` - The sync ID for client-server matching
    /// - `player_inventory` - The player's inventory
    /// - `inventory` - The brewing stand's inventory (5 slots)
    /// - `property_delegate` - Delegate for accessing brew time and fuel properties
    pub async fn new(
        sync_id: u8,
        player_inventory: Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        property_delegate: Arc<dyn PropertyDelegate>,
    ) -> Self {
        struct BrewingScreenListener;
        impl crate::screen_handler::ScreenHandlerListener for BrewingScreenListener {
            fn on_property_update<'a>(
                &'a self,
                screen_handler: &'a ScreenHandlerBehaviour,
                property: u8,
                value: i32,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
                Box::pin(async move {
                    if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                        sync_handler
                            .update_property(screen_handler, i32::from(property), value)
                            .await;
                    }
                })
            }
        }

        let mut handler = Self {
            inventory,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::BrewingStand)),
            _property_delegate: property_delegate.clone(),
        };

        // BrewTime (index 0) and Fuel (index 1)
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 0));
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 1));

        handler.add_listener(Arc::new(BrewingScreenListener)).await;

        // Add all 5 brewing stand slots: 0-2 = potions, 3 = ingredient, 4 = fuel
        for i in 0..5 {
            handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
                handler.inventory.clone(),
                i,
            )));
        }

        // Add player slots
        let pi: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&pi);

        handler
    }
}

impl ScreenHandler for BrewingScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn crate::screen_handler::InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move { self.default_on_closed(player).await })
    }

    /// Quick move logic for brewing stand.
    ///
    /// - Potions (0-2) -> Player inventory
    /// - Ingredient (3) -> Player inventory
    /// - Fuel (4) -> Player inventory
    /// - From player: potions -> potion slots, fuel -> fuel slot, else -> ingredient slot
    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn crate::screen_handler::InventoryPlayer,
        slot_index: i32,
    ) -> Pin<Box<dyn Future<Output = ItemStack> + Send + 'a>> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();

            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if !slot.has_stack().await {
                return stack_left;
            }

            let slot_stack_lock = slot.get_stack().await;
            let mut stack = slot_stack_lock.lock().await;
            stack_left = stack.clone();

            let success = if slot_index < 5 {
                // Moving from brewing stand to player inventory
                self.insert_item(&mut stack, 5, self.get_behaviour().slots.len() as i32, true)
                    .await
            } else {
                // Moving from player inventory to brewing stand
                // Check item type to determine target slot

                // Check if item has potion contents (for slots 0-2)
                let has_potion_contents = stack
                    .get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>()
                    .is_some();

                // Check if item is brewing fuel (for slot 4)
                let is_fuel = stack.get_item().has_tag(&tag::Item::MINECRAFT_BREWING_FUEL);

                if has_potion_contents {
                    // Try to insert into potion slots (0-2)
                    self.insert_item(&mut stack, 0, 3, false).await
                } else if is_fuel {
                    // Try to insert into fuel slot (4)
                    self.insert_item(&mut stack, 4, 5, false).await
                } else {
                    // Try to insert into ingredient slot (3)
                    self.insert_item(&mut stack, 3, 4, false).await
                }
            };

            if !success {
                return ItemStack::EMPTY.clone();
            }

            if stack.is_empty() {
                drop(stack);
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.mark_dirty().await;
            }

            stack_left
        })
    }
}

/// Creates a new brewing stand screen handler.
///
/// Factory function used by the server when a player opens a brewing stand.
pub async fn create_brewing(
    sync_id: u8,
    player_inventory: Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
    property_delegate: Arc<dyn PropertyDelegate>,
) -> Option<impl ScreenHandler> {
    let handler =
        BrewingScreenHandler::new(sync_id, player_inventory, inventory, property_delegate).await;
    Some(handler)
}
