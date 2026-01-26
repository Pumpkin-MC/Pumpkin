use pumpkin_data::packet::clientbound::PLAY_TICKING_STEP;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_TICKING_STEP)]
pub struct CTickingStep {
    pub tick_steps: VarInt,
}

impl CTickingStep {
    pub fn new(tick_steps: VarInt) -> Self {
        Self { tick_steps }
    }
}
