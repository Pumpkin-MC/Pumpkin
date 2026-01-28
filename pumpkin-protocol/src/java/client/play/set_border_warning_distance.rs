use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DISTANCE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_SET_BORDER_WARNING_DISTANCE)]
pub struct CSetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    #[must_use]
    pub fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}
