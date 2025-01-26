use pumpkin_data::packet::clientbound::PLAY_CLEAR_TITLES;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_CLEAR_TITLES)]
pub struct CClearTtitle {
    reset: bool,
}

impl CClearTtitle {
    pub const fn new(reset: bool) -> Self {
        Self { reset }
    }
}
