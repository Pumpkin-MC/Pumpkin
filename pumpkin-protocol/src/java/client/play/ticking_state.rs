use pumpkin_data::packet::clientbound::PLAY_TICKING_STATE;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_TICKING_STATE)]
pub struct CTickingState {
    tick_rate: f32,
    is_frozen: bool,
}

impl CTickingState {
    pub fn new(tick_rate: f32, is_frozen: bool) -> Self {
        Self {
            tick_rate,
            is_frozen,
        }
    }
}
