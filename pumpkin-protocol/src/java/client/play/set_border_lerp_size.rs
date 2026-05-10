use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_LERP_SIZE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::codec::var_long::VarLong;

#[derive(Serialize)]
#[java_packet(PLAY_SET_BORDER_LERP_SIZE)]
pub struct CSetBorderLerpSize {
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: VarLong,
}

impl CSetBorderLerpSize {
    #[must_use]
    pub const fn new(old_diameter: f64, new_diameter: f64, speed: VarLong) -> Self {
        Self {
            old_diameter,
            new_diameter,
            speed,
        }
    }
}
