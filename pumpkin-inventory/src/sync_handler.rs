use std::sync::Arc;

use pumpkin_protocol::{
    codec::{
        item_stack_seralizer::{ItemStackSerializer, OptionalItemStackHash},
        var_int::VarInt,
    },
    java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
    },
};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::screen_handler::{InventoryPlayer, ScreenHandlerBehaviour};

pub struct SyncHandler {
    player: Mutex<Option<Arc<dyn InventoryPlayer>>>,
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncHandler {
    pub fn new() -> Self {
        Self {
            player: Mutex::new(None),
        }
    }

    pub async fn store_player(&self, player: Arc<dyn InventoryPlayer>) {
        self.player.lock().await.replace(player);
    }

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
                        .map(|stack| ItemStackSerializer::from(*stack))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &ItemStackSerializer::from(*cursor_stack),
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
                    &ItemStackSerializer::from(*stack),
                ))
                .await;
        }
    }

    pub async fn update_cursor_stack(
        &self,
        _screen_handler: &ScreenHandlerBehaviour,
        stack: &ItemStack,
    ) {
        if let Some(player) = self.player.lock().await.as_ref() {
            player
                .enqueue_cursor_packet(&CSetCursorItem::new(&ItemStackSerializer::from(*stack)))
                .await;
        }
    }

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

// TrackedSlot in vanilla
#[derive(Debug, Clone)]
pub struct TrackedStack {
    pub received_stack: Option<ItemStack>,
    pub received_hash: Option<OptionalItemStackHash>,
}

impl TrackedStack {
    pub const EMPTY: TrackedStack = TrackedStack {
        received_stack: None,
        received_hash: None,
    };

    pub fn set_received_stack(&mut self, stack: ItemStack) {
        self.received_stack = Some(stack);
        self.received_hash = None;
    }

    pub fn set_received_hash(&mut self, hash: OptionalItemStackHash) {
        self.received_hash = Some(hash);
        self.received_stack = None;
    }

    pub fn is_in_sync(&mut self, actual_stack: &ItemStack) -> bool {
        if let Some(stack) = &self.received_stack {
            return stack.are_equal(actual_stack);
        } else if let Some(hash) = &self.received_hash {
            if hash.hash_equals(actual_stack) {
                self.received_stack = Some(*actual_stack);
                return true;
            }
        }

        false
    }
}
