use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_CENTER;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_BORDER_CENTER)]
pub struct CSetBorderCenter {
    pub x: f64,
    pub z: f64,
}

impl CSetBorderCenter {
    #[must_use]
    pub fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }
}
