//! Furnace-like screen handler.
//!
//! This module implements the screen handler for furnace-like blocks:
//! - Furnace
//! - Smoker
//! - Blast Furnace
//!
//! All three share the same 3-slot layout:
//! - Slot 0: Input (item to smelt/cook)
//! - Slot 1: Fuel (coal, charcoal, etc.)
//! - Slot 2: Output (smelted result)
//!
//! The screen handler tracks 4 properties:
//! - Property 0: Fire icon animation (fuel burn time remaining)
//! - Property 1: Maximum fuel burn time
//! - Property 2: Progress arrow (cooking/smelt time)
//! - Property 3: Maximum progress (typically 200 ticks for furnace)

use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::{fuels::is_fuel, item_stack::ItemStack, screen::WindowType};
use pumpkin_world::{
    block::entities::{ExperienceContainer, PropertyDelegate},
    inventory::Inventory,
};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, ScreenHandlerListener, ScreenProperty,
    },
};
use tracing::debug;

use super::furnace_like_slot::{FurnaceLikeSlot, FurnaceLikeSlotType, FurnaceOutputSlot};

/// Screen handler for furnace-like containers.
///
/// Handles the UI for furnaces, smokers, and blast furnaces.
/// These all share the same slot layout and quick-move behavior.
pub struct FurnaceLikeScreenHandler {
    /// The furnace's inventory (3 slots: 0 input, 1 fuel, 2 output).
    pub inventory: Arc<dyn Inventory>,
    /// Container that tracks accumulated smelting experience.
    ///
    /// Experience is awarded to the player when they take items from the output slot.
    experience_container: Arc<dyn ExperienceContainer>,
    /// Core screen handler behavior (slots, sync ID, properties, listeners).
    behaviour: ScreenHandlerBehaviour,
}

impl FurnaceLikeScreenHandler {
    /// Creates a new furnace-like screen handler.
    ///
    /// # Arguments
    /// - `sync_id` - The sync ID for client-server matching
    /// - `player_inventory` - The player's inventory
    /// - `inventory` - The furnace's inventory (3 slots)
    /// - `property_delegate` - Delegate for accessing furnace properties
    /// - `experience_container` - Container that tracks smelting experience
    /// - `window_type` - The window type (Furnace, Smoker, or `BlastFurnace`)
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        property_delegate: Arc<dyn PropertyDelegate>,
        experience_container: Arc<dyn ExperienceContainer>,
        window_type: WindowType,
    ) -> Self {
        struct FurnaceLikeScreenListener;
        impl ScreenHandlerListener for FurnaceLikeScreenListener {
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
            experience_container,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(window_type)),
        };

        // 0: Fire icon (fuel left) counting from fuel burn time down to 0 (in-game ticks)
        // 1: Maximum fuel burn time fuel burn time or 0 (in-game ticks)
        // 2: Progress arrow counting from 0 to maximum progress (in-game ticks)
        // 3: Maximum progress always 200 on the vanilla server
        for i in 0..4 {
            handler.add_property(ScreenProperty::new(property_delegate.clone(), i));
        }

        handler
            .add_listener(Arc::new(FurnaceLikeScreenListener))
            .await;
        handler.add_inventory_slots();
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    /// Adds the 3 furnace inventory slots.
    ///
    /// - Slot 0: Input (top)
    /// - Slot 1: Fuel (bottom)
    /// - Slot 2: Output
    fn add_inventory_slots(&mut self) {
        self.add_slot(Arc::new(FurnaceLikeSlot::new(
            self.inventory.clone(),
            FurnaceLikeSlotType::Top,
        )));
        self.add_slot(Arc::new(FurnaceLikeSlot::new(
            self.inventory.clone(),
            FurnaceLikeSlotType::Bottom,
        )));
        // Output slot awards experience when items are taken
        self.add_slot(Arc::new(FurnaceOutputSlot::new(
            self.inventory.clone(),
            self.experience_container.clone(),
        )));
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

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // TODO: self.inventory.on_closed(player).await;
        })
    }

    /// Quick move logic for furnace-like containers.
    ///
    /// - From furnace slots (0-2): Move to player inventory
    /// - Fuel items: Move to fuel slot (1)
    /// - Other items: Move to input slot (0)
    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            const FUEL_SLOT: i32 = 1; // Note: Slots 0, 1, 2 are Furnace slots.
            const OUTPUT_SLOT: i32 = 2;

            debug!("FurnaceLikeScreenHandler::quick_move slot_index={slot_index}");

            let mut stack_left = ItemStack::EMPTY.clone();

            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if !slot.has_stack().await {
                return stack_left;
            }

            let slot_stack_lock = slot.get_stack().await;

            // Acquire the lock to read/clone the stack
            let mut stack = slot_stack_lock.lock().await;
            stack_left = stack.clone();

            let success = if slot_index < 3 {
                // If clicked slot is one of the Furnace slots (0, 1, 2):
                // Try to move to player inventory (slots 3 onwards, starting from the end)
                self.insert_item(&mut stack, 3, self.get_behaviour().slots.len() as i32, true)
                    .await
            } else if is_fuel(stack.item.id) {
                // If clicked slot is in the player inventory (3+) and contains fuel:
                // Try to move to the Furnace's Fuel slot (slot 1)
                self.insert_item(&mut stack, FUEL_SLOT, 3, false).await
            } else {
                // If clicked slot is in the player inventory (3+) and NOT fuel (must be a smeltable item):
                // Try to move to the Furnace's Input/Smelting slot (slot 0)
                self.insert_item(&mut stack, 0, 3, false).await
            };

            if !success {
                return ItemStack::EMPTY.clone();
            }

            if stack.is_empty() {
                drop(stack); // Release lock before awaiting
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.mark_dirty().await;
            }

            // Award XP when taking from output slot (slot 2)
            if slot_index == OUTPUT_SLOT {
                debug!("quick_move: taking from output slot, calling on_take_item");
                slot.on_take_item(player, &stack_left).await;
            }

            stack_left
        })
    }
}
