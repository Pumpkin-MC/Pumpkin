use std::sync::Arc;

use pumpkin_protocol::{
    client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
    },
    codec::{item_stack_seralizer::ItemStackSerializer, var_int::VarInt},
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
                .enque_inventory_packet(&CSetContainerContent::new(
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
                    .enque_property_packet(&CSetContainerProperty::new(
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
                .enque_slot_packet(&CSetContainerSlot::new(
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
                .enque_cursor_packet(&CSetCursorItem::new(&ItemStackSerializer::from(*stack)))
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
                .enque_property_packet(&CSetContainerProperty::new(
                    VarInt(screen_handler.sync_id.into()),
                    property as i16,
                    value as i16,
                ))
                .await;
        }
    }
}
