use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_CENTER;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
