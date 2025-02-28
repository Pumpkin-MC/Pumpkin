use pumpkin_data::packet::clientbound::PLAY_CLEAR_TITLES;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CLEAR_TITLES)]
pub struct CClearTtitle {
    reset: bool,
}

impl CClearTtitle {
    pub const fn new(reset: bool) -> Self {
        Self { reset }
    }
}
