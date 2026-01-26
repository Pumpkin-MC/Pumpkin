use pumpkin_data::packet::clientbound::PLAY_TICKING_STATE;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_TICKING_STATE)]
pub struct CTickingState {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

impl CTickingState {
    pub fn new(tick_rate: f32, is_frozen: bool) -> Self {
        Self {
            tick_rate,
            is_frozen,
        }
    }
}
