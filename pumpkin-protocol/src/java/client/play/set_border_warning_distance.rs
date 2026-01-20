use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DISTANCE;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_WARNING_DISTANCE)]
pub struct CSetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    #[must_use] 
    pub const fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}
