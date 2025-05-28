use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_CENTER;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_CENTER)]
pub struct CSetBorderCenter {
    x: f64,
    z: f64,
}

impl CSetBorderCenter {
    pub fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }
}
