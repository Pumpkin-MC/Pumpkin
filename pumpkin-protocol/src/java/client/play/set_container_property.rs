use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_CONTAINER_SET_DATA)]
pub struct CSetContainerProperty {
    window_id: VarInt,
    property: i16,
    value: i16,
}

impl CSetContainerProperty {
    pub const fn new(window_id: VarInt, property: i16, value: i16) -> Self {
        Self {
            window_id,
            property,
            value,
        }
    }
}
