use pumpkin_data::packet::clientbound::PLAY_FORGET_LEVEL_CHUNK;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_FORGET_LEVEL_CHUNK)]
pub struct CUnloadChunk {
    z: i32,
    x: i32,
}

impl CUnloadChunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}
