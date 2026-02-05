use pumpkin_data::packet::clientbound::PLAY_SET_PASSENGERS;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_SET_PASSENGERS)]
pub struct CSetPassengers<'a> {
    pub entity_id: VarInt,
    pub passengers: &'a [VarInt],
}

impl<'a> CSetPassengers<'a> {
    #[must_use]
    pub const fn new(entity_id: VarInt, passengers: &'a [VarInt]) -> Self {
        Self {
            entity_id,
            passengers,
        }
    }
}
