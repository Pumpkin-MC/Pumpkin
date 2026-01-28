use pumpkin_data::packet::clientbound::PLAY_FORGET_LEVEL_CHUNK;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_FORGET_LEVEL_CHUNK)]
pub struct CUnloadChunk {
    pub z: i32,
    pub x: i32,
}

impl CUnloadChunk {
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}
