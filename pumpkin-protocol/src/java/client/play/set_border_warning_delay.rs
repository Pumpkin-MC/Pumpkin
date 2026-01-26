use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DELAY;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_SET_BORDER_WARNING_DELAY)]
pub struct CSetBorderWarningDelay {
    pub warning_time: VarInt,
}

impl CSetBorderWarningDelay {
    pub fn new(warning_time: VarInt) -> Self {
        Self { warning_time }
    }
}
