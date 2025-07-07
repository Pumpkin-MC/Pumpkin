use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DISTANCE;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_WARNING_DISTANCE)]
pub struct CSetBorderWarningDistance {
    warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    pub fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}
