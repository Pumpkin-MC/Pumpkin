use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_SET_PLAYER_INVENTORY;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_PLAYER_INVENTORY)]
pub struct CSetPlayerInventory<'a> {
    pub slot: VarInt,
    pub item: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetPlayerInventory<'a> {
    #[must_use]
    pub const fn new(slot: VarInt, item: &'a ItemStackSerializer<'a>) -> Self {
        Self { slot, item }
    }
}
