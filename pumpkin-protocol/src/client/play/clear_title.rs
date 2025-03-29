use pumpkin_data::packet::clientbound::PLAY_CLEAR_TITLES;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_CLEAR_TITLES)]
pub struct CClearTitle {
    reset: bool,
}

impl CClearTitle {
    pub const fn new(reset: bool) -> Self {
        Self { reset }
    }
}
