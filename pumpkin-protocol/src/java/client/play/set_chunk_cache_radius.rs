use pumpkin_data::packet::clientbound::PLAY_SET_CHUNK_CACHE_RADIUS;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Tells the client the server's view distance (chunk loading radius).
#[derive(Serialize)]
#[java_packet(PLAY_SET_CHUNK_CACHE_RADIUS)]
pub struct CSetChunkCacheRadius {
    pub radius: VarInt,
}

impl CSetChunkCacheRadius {
    #[must_use]
    pub const fn new(radius: VarInt) -> Self {
        Self { radius }
    }
}
