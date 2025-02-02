use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DELAY;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_SET_BORDER_WARNING_DELAY)]
pub struct CSetBorderWarningDelay {
    warning_time: VarInt,
}

impl CSetBorderWarningDelay {
    pub fn new(warning_time: VarInt) -> Self {
        Self { warning_time }
    }
}
