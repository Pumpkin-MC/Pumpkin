use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_SET_PLAYER_INVENTORY;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_PLAYER_INVENTORY)]
pub struct CSetPlayerInventory<'a> {
    slot: VarInt,
    item: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetPlayerInventory<'a> {
    pub fn new(slot: VarInt, item: &'a ItemStackSerializer<'a>) -> Self {
        Self { slot, item }
    }
}
