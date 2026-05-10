//! Inventory synchronization handler.
//!
//! This module handles the synchronization of inventory state between the server
//! and connected clients. It ensures that players see the correct items in slots,
//! cursor items, and container properties (like furnace progress).
//!
//! # Synchronization
//!
//! The sync handler manages:
//! - Full container content updates (sent when opening a container or on major changes)
//! - Individual slot updates (sent when a single slot changes)
//! - Cursor item updates (the item being held by the mouse cursor)
//! - Property updates (container-specific data like furnace burn time)
//!
//! # Revision Tracking
//!
//! Each synchronization message includes a revision number to ensure the client
//! and server stay in sync. If the client detects a desync, it can request a full
//! resynchronization.

use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::{
    codec::{
        item_stack_seralizer::{ItemStackSerializer, OptionalItemStackHash},
        var_int::VarInt,
    },
    java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
    },
};
use tokio::sync::Mutex;

use crate::screen_handler::{InventoryPlayer, ScreenHandlerBehaviour};

/// Handles inventory synchronization to a specific player.
///
/// The sync handler stores a reference to the player and sends inventory
/// update packets when container state changes. It manages:
/// - Full content synchronization
/// - Incremental slot updates
/// - Cursor item tracking
/// - Property (UI element) updates
pub struct SyncHandler {
    /// The player to synchronize inventory updates with.
    ///
    /// None until `store_player` is called to attach a player.
    player: Mutex<Option<Arc<dyn InventoryPlayer>>>,
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncHandler {
    /// Creates a new sync handler with no player attached.
    #[must_use]
    pub fn new() -> Self {
        Self {
            player: Mutex::new(None),
        }
    }

    /// Stores the player to synchronize with.
    ///
    /// Must be called before any sync operations.
    pub async fn store_player(&self, player: Arc<dyn InventoryPlayer>) {
        self.player.lock().await.replace(player);
    }

    /// Sends a full container content update.
    ///
    /// This sends all slots, the cursor item, and properties to the client.
    /// Used for initial sync and recovery from desync.
    ///
    /// # Arguments
    /// - `screen_handler` - The screen handler to sync
    /// - `stacks` - All slot contents
    /// - `cursor_stack` - The item held by the cursor
    /// - `properties` - Container property values
    /// - `next_revision` - The new revision number
    pub async fn update_state(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        stacks: &[ItemStack],
        cursor_stack: &ItemStack,
        properties: Vec<i32>,
        next_revision: u32,
    ) {
        if let Some(player) = self.player.lock().await.as_ref() {
            player
                .enqueue_inventory_packet(&CSetContainerContent::new(
                    VarInt(screen_handler.sync_id.into()),
                    VarInt(next_revision as i32),
                    stacks
                        .iter()
                        .map(|stack| ItemStackSerializer::from(stack.clone()))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &ItemStackSerializer::from(cursor_stack.clone()),
                ))
                .await;

            for (i, property) in properties.iter().enumerate() {
                player
                    .enqueue_property_packet(&CSetContainerProperty::new(
                        VarInt(screen_handler.sync_id.into()),
                        i as i16,
                        *property as i16,
                    ))
                    .await;
            }
        }
    }

    /// Updates a single slot on the client.
    ///
    /// More efficient than full sync for single-slot changes.
    ///
    /// # Arguments
    /// - `screen_handler` - The screen handler
    /// - `slot` - The slot index that changed
    /// - `stack` - The new stack in that slot
    /// - `next_revision` - The new revision number
    pub async fn update_slot(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        slot: usize,
        stack: &ItemStack,
        next_revision: u32,
    ) {
        if let Some(player) = self.player.lock().await.as_ref() {
            player
                .enqueue_slot_packet(&CSetContainerSlot::new(
                    screen_handler.sync_id as i8,
                    next_revision as i32,
                    slot as i16,
                    &ItemStackSerializer::from(stack.clone()),
                ))
                .await;
        }
    }

    /// Updates the cursor item on the client.
    ///
    /// Sent when the player's held (cursor) item changes.
    ///
    /// # Arguments
    /// - `screen_handler` - The screen handler
    /// - `stack` - The new cursor item
    pub async fn update_cursor_stack(
        &self,
        _screen_handler: &ScreenHandlerBehaviour,
        stack: &ItemStack,
    ) {
        if let Some(player) = self.player.lock().await.as_ref() {
            player
                .enqueue_cursor_packet(&CSetCursorItem::new(&ItemStackSerializer::from(
                    stack.clone(),
                )))
                .await;
        }
    }

    /// Updates a container property on the client.
    ///
    /// Used for UI elements like furnace progress bars.
    ///
    /// # Arguments
    /// - `screen_handler` - The screen handler
    /// - `property` - The property index
    /// - `value` - The new property value
    pub async fn update_property(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        property: i32,
        value: i32,
    ) {
        if let Some(player) = self.player.lock().await.as_ref() {
            player
                .enqueue_property_packet(&CSetContainerProperty::new(
                    VarInt(screen_handler.sync_id.into()),
                    property as i16,
                    value as i16,
                ))
                .await;
        }
    }
}

/// Tracks the last known state of a slot for sync purposes.
///
/// Used to detect when a slot has changed and needs to be synced to the client.
/// Stores either the full stack or a hash for comparison.
#[derive(Clone)]
pub struct TrackedStack {
    /// The full item stack last sent to the client.
    ///
    /// Set when sending full stack data. Cleared when only sending a hash.
    pub received_stack: Option<ItemStack>,
    /// The hash of the item stack last sent to the client.
    ///
    /// Used for lightweight comparison to detect changes.
    pub received_hash: Option<OptionalItemStackHash>,
}

impl TrackedStack {
    /// An empty tracked stack with no known state.
    pub const EMPTY: Self = Self {
        received_stack: None,
        received_hash: None,
    };

    /// Records that we sent this stack to the client.
    pub fn set_received_stack(&mut self, stack: ItemStack) {
        self.received_stack = Some(stack);
        self.received_hash = None;
    }

    /// Records that we sent this hash to the client.
    pub fn set_received_hash(&mut self, hash: OptionalItemStackHash) {
        self.received_hash = Some(hash);
        self.received_stack = None;
    }

    /// Checks if the actual stack matches our tracked state.
    ///
    /// Updates the tracked state to the actual stack if they match.
    //FIX Methods named `is_*` normally take self by reference or no self. Consider choosing a less ambiguous name.
    pub fn is_in_sync(&mut self, actual_stack: &ItemStack) -> bool {
        if let Some(stack) = &self.received_stack {
            return stack.are_equal(actual_stack);
        } else if let Some(hash) = &self.received_hash
            && hash.hash_equals(actual_stack)
        {
            self.received_stack = Some(actual_stack.clone());
            return true;
        }

        false
    }
}
