use pumpkin_data::packets::clientbound::PLAY_SET_HELD_SLOT;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_SET_HELD_SLOT)]
pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}
