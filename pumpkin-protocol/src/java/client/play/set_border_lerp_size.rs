use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_LERP_SIZE;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_long::VarLong;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_LERP_SIZE)]
pub struct CSetBorderLerpSize {
    old_diameter: f64,
    new_diameter: f64,
    speed: VarLong,
}

impl CSetBorderLerpSize {
    pub fn new(old_diameter: f64, new_diameter: f64, speed: VarLong) -> Self {
        Self {
            old_diameter,
            new_diameter,
            speed,
        }
    }
}
