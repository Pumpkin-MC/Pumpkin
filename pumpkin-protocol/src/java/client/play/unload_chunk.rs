use pumpkin_data::packet::clientbound::PLAY_FORGET_LEVEL_CHUNK;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_FORGET_LEVEL_CHUNK)]
pub struct CUnloadChunk {
    pub z: i32,
    pub x: i32,
}

impl CUnloadChunk {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}
