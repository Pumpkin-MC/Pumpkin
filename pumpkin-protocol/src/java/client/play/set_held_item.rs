use pumpkin_data::packet::clientbound::PLAY_SET_HELD_SLOT;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_HELD_SLOT)]
pub struct CSetSelectedSlot {
    pub slot: i8,
}

impl CSetSelectedSlot {
    #[must_use]
    pub const fn new(slot: i8) -> Self {
        Self { slot }
    }
}
