use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_SIZE;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_BORDER_SIZE)]
pub struct CSetBorderSize {
    pub diameter: f64,
}

impl CSetBorderSize {
    #[must_use] 
    pub const fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}
