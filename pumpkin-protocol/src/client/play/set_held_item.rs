use pumpkin_data::packet::clientbound::PLAY_SET_HELD_SLOT;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_SET_HELD_SLOT)]
pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}
