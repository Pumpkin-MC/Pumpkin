use pumpkin_data::packet::clientbound::PLAY_CLEAR_TITLES;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CLEAR_TITLES)]
pub struct CClearTitle {
    pub reset: bool,
}

impl CClearTitle {
    #[must_use] 
    pub const fn new(reset: bool) -> Self {
        Self { reset }
    }
}
