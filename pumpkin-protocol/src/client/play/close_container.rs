use pumpkin_data::packet::clientbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_CONTAINER_CLOSE)]
pub struct CCloseContainer {
    window_id: VarInt,
}

impl CCloseContainer {
    pub const fn new(window_id: VarInt) -> Self {
        Self { window_id }
    }
}
