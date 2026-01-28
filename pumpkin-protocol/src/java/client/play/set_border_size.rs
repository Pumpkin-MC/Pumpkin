use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_SIZE;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_BORDER_SIZE)]
pub struct CSetBorderSize {
    pub diameter: f64,
}

impl CSetBorderSize {
    #[must_use]
    pub fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}
