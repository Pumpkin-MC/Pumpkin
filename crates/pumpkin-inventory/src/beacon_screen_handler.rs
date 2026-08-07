use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::{block::entities::PropertyDelegate, inventory::Inventory};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, ScreenProperty,
    },
    slot::NormalSlot,
};

/// Creates a beacon container screen handler.
///
/// Beacons feature a single payment slot and a specialized UI for selecting status effects.
pub async fn create_beacon_handler(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
    property_delegate: Arc<dyn PropertyDelegate>,
) -> BeaconScreenHandler {
    BeaconScreenHandler::new(sync_id, player_inventory, inventory, property_delegate).await
}

/// Screen handler specifically for Beacon blocks.
pub struct BeaconScreenHandler {
    /// The beacon's inventory (contains exactly 1 slot for payment).
    pub inventory: Arc<dyn Inventory>,
    /// Core screen handler behavior (slots, sync ID, listeners).
    behaviour: ScreenHandlerBehaviour,
    /// Delegate for the levels/primary/secondary properties synced to the client.
    _property_delegate: Arc<dyn PropertyDelegate>,
}

impl BeaconScreenHandler {
    /// Creates a new beacon screen handler.
    async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        property_delegate: Arc<dyn PropertyDelegate>,
    ) -> Self {
        struct BeaconScreenListener;
        impl crate::screen_handler::ScreenHandlerListener for BeaconScreenListener {
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
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Beacon)),
            _property_delegate: property_delegate.clone(),
        };

        inventory.on_open().await;

        // Levels (index 0), primary effect (index 1), secondary effect (index 2)
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 0));
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 1));
        handler.add_property(ScreenProperty::new(property_delegate.clone(), 2));

        handler.add_listener(Arc::new(BeaconScreenListener)).await;

        // Add the single payment slot for the beacon (slot 0)
        handler.add_slot(Arc::new(NormalSlot::new(handler.inventory.clone(), 0)));

        // Add the player's inventory slots (27 slots + 9 hotbar)
        let player_inventory_arc: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory_arc);

        handler
    }
}

impl ScreenHandler for BeaconScreenHandler {
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
            self.inventory.on_close().await;
        })
    }

    /// Quick move logic specifically for the beacon UI.
    ///
    /// - From beacon payment slot (0): Move to player inventory
    /// - From player inventory (1+): Move to beacon payment slot
    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                drop(slot_stack_guard);

                let mut slot_stack_mut = slot_stack_lock.lock().await;

                if slot_index == 0 {
                    // Move from the single beacon slot to the player inventory (slots 1 to end)
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            1,
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // Move from player inventory into the beacon payment slot (slot 0)
                    if !self.insert_item(&mut slot_stack_mut, 0, 1, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                }

                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut);
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut);
                    slot.mark_dirty().await;
                }
            }

            stack_left
        })
    }
}
