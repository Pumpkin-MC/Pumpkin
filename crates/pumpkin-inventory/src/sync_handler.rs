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

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_protocol::{
    codec::{
        item_stack_seralizer::{ItemStackSerializer, OptionalItemStackHash},
        var_int::VarInt,
    },
    java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
    },
};

use crate::screen_handler::{InventoryPlayer, ScreenHandlerBehaviour, SharedScreenHandler};

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
    stack_network_ids: AtomicBool,
    deferred: Option<Mutex<Vec<ScreenUpdate>>>,
}

enum ScreenUpdate {
    Content(
        u8,
        u32,
        Option<WindowType>,
        Vec<ItemStackSerializer<'static>>,
        ItemStack,
    ),
    Slot(u8, u32, usize, ItemStack, Option<WindowType>, usize),
    Cursor(ItemStack),
    Property(u8, i16, i16, Option<WindowType>),
}

/// State the deferred packet describes. Locks are acquired only outside
/// packet callbacks and remain held until the packet is queued.
#[derive(Clone, Copy)]
pub struct ScreenUpdateContext<'a> {
    pub screen: &'a SharedScreenHandler,
    pub active: Option<&'a Mutex<SharedScreenHandler>>,
    pub epoch: u64,
}

pub struct ScreenUpdateGuard<'a> {
    _active: Option<std::sync::MutexGuard<'a, SharedScreenHandler>>,
    _screen: std::sync::MutexGuard<'a, dyn crate::screen_handler::ScreenHandler + 'static>,
}

impl<'a> ScreenUpdateContext<'a> {
    #[must_use]
    pub fn lock_if_current(&self) -> Option<ScreenUpdateGuard<'a>> {
        self.lock_for_delivery(|| Some(()))
            .map(|(screen, ())| screen)
    }

    /// Also verifies physical slots shared with other menus while the screen stays locked.
    pub fn lock_for_delivery<T>(
        &self,
        validate: impl FnOnce() -> Option<T>,
    ) -> Option<(ScreenUpdateGuard<'a>, T)> {
        let (active, mut screen) = loop {
            let active = self.active.map(|active| {
                active
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            });
            if active
                .as_ref()
                .is_some_and(|active| !Arc::ptr_eq(self.screen, active))
            {
                return None;
            }
            match self.screen.try_lock() {
                Ok(screen) => break (active, screen),
                Err(std::sync::TryLockError::Poisoned(error)) => {
                    break (active, error.into_inner());
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    drop(active);
                    drop(
                        self.screen
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    );
                }
            }
        };
        if screen.get_behaviour().sync_epoch() != self.epoch {
            // Capturing packets also updates the expected client state. Reconcile
            // the whole menu next time it is synchronized if any packet is superseded.
            screen.get_behaviour_mut().needs_full_sync = true;
            return None;
        }
        let Some(physical) = validate() else {
            screen.get_behaviour_mut().needs_full_sync = true;
            return None;
        };
        Some((
            ScreenUpdateGuard {
                _active: active,
                _screen: screen,
            },
            physical,
        ))
    }
}

/// A screen update already adapted to the Java inventory packet model.
pub enum ScreenPacket<'a> {
    Content(CSetContainerContent<'a>, Option<WindowType>),
    Slot(CSetContainerSlot<'a>, Option<WindowType>, usize),
    Cursor(CSetCursorItem<'a>),
    Property(CSetContainerProperty, Option<WindowType>),
}

impl ScreenUpdate {
    fn send(self, player: &dyn InventoryPlayer, is_current: Option<ScreenUpdateContext<'_>>) {
        match self {
            Self::Content(id, revision, window, slots, cursor) => player.enqueue_screen_packet(
                ScreenPacket::Content(
                    CSetContainerContent::new(
                        VarInt(id.into()),
                        VarInt(revision as i32),
                        &slots,
                        &ItemStackSerializer::from(cursor),
                    ),
                    window,
                ),
                is_current,
            ),
            Self::Slot(id, revision, slot, stack, window, total) => player.enqueue_screen_packet(
                ScreenPacket::Slot(
                    CSetContainerSlot::new(
                        id as i8,
                        revision as i32,
                        slot as i16,
                        &ItemStackSerializer::from(stack),
                    ),
                    window,
                    total,
                ),
                is_current,
            ),
            Self::Cursor(stack) => player.enqueue_screen_packet(
                ScreenPacket::Cursor(CSetCursorItem::new(&ItemStackSerializer::from(stack))),
                is_current,
            ),
            Self::Property(id, property, value, window) => player.enqueue_screen_packet(
                ScreenPacket::Property(
                    CSetContainerProperty::new(VarInt(id.into()), property, value),
                    window,
                ),
                is_current,
            ),
        }
    }
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
            stack_network_ids: AtomicBool::new(false),
            deferred: None,
        }
    }

    pub(crate) fn deferred(&self) -> Self {
        Self {
            player: Mutex::new(self.player()),
            stack_network_ids: AtomicBool::new(self.uses_stack_network_ids()),
            deferred: Some(Mutex::new(Vec::new())),
        }
    }

    fn player(&self) -> Option<Arc<dyn InventoryPlayer>> {
        self.player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn dispatch(&self, screen: &ScreenHandlerBehaviour, packet: ScreenUpdate) {
        screen.invalidate_sync();
        if let Some(deferred) = &self.deferred {
            deferred
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(packet);
        } else if let Some(player) = self.player() {
            packet.send(player.as_ref(), None);
        }
    }

    pub(crate) fn uses_stack_network_ids(&self) -> bool {
        self.stack_network_ids.load(Ordering::Relaxed)
    }

    /// Flushes a captured screen update after releasing the screen handler.
    pub fn flush(&self, context: ScreenUpdateContext<'_>) {
        let Some(deferred) = &self.deferred else {
            return;
        };
        let packets = std::mem::take(
            &mut *deferred
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if let Some(player) = self.player() {
            for packet in packets {
                if context.lock_if_current().is_none() {
                    break;
                }
                packet.send(player.as_ref(), Some(context));
            }
        }
    }

    /// Stores the player to synchronize with.
    ///
    /// Must be called before any sync operations.
    pub fn store_player(&self, player: Arc<dyn InventoryPlayer>) {
        self.stack_network_ids
            .store(player.uses_stack_network_ids(), Ordering::Relaxed);
        self.player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(player);
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
    pub fn update_state(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        stacks: &[ItemStack],
        cursor_stack: &ItemStack,
        properties: &[i32],
        next_revision: u32,
    ) {
        self.dispatch(
            screen_handler,
            ScreenUpdate::Content(
                screen_handler.sync_id,
                next_revision,
                screen_handler.window_type,
                stacks
                    .iter()
                    .cloned()
                    .map(ItemStackSerializer::from)
                    .collect(),
                cursor_stack.clone(),
            ),
        );
        for (i, property) in properties.iter().enumerate() {
            self.update_property(screen_handler, i as i32, *property);
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
    pub fn update_slot(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        slot: usize,
        stack: &ItemStack,
        next_revision: u32,
    ) {
        self.dispatch(
            screen_handler,
            ScreenUpdate::Slot(
                screen_handler.sync_id,
                next_revision,
                slot,
                stack.clone(),
                screen_handler.window_type,
                screen_handler.slots.len(),
            ),
        );
    }

    /// Updates the cursor item on the client.
    ///
    /// Sent when the player's held (cursor) item changes.
    ///
    /// # Arguments
    /// - `stack` - The new cursor item
    pub fn update_cursor_stack(&self, screen_handler: &ScreenHandlerBehaviour, stack: &ItemStack) {
        self.dispatch(screen_handler, ScreenUpdate::Cursor(stack.clone()));
    }

    /// Updates a container property on the client.
    ///
    /// Used for UI elements like furnace progress bars.
    ///
    /// # Arguments
    /// - `screen_handler` - The screen handler
    /// - `property` - The property index
    /// - `value` - The new property value
    pub fn update_property(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        property: i32,
        value: i32,
    ) {
        self.dispatch(
            screen_handler,
            ScreenUpdate::Property(
                screen_handler.sync_id,
                property as i16,
                value as i16,
                screen_handler.window_type,
            ),
        );
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
        self.is_in_sync_with_identity(actual_stack, false)
    }

    pub(crate) fn is_in_sync_with_identity(
        &mut self,
        actual_stack: &ItemStack,
        compare_uid: bool,
    ) -> bool {
        if let Some(stack) = &self.received_stack {
            return (!compare_uid || actual_stack.is_empty() || stack.uid == actual_stack.uid)
                && stack.are_equal(actual_stack);
        } else if let Some(hash) = &self.received_hash
            && hash.hash_equals(actual_stack)
        {
            self.received_stack = Some(actual_stack.clone());
            return true;
        }

        false
    }
}
