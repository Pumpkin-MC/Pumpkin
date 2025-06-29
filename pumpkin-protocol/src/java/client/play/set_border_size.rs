use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_SIZE;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_SIZE)]
pub struct CSetBorderSize {
    diameter: f64,
}

impl CSetBorderSize {
    pub fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}
