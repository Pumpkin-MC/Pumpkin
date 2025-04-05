use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_FINISHED;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

#[derive(Serialize)]
#[packet(PLAY_CHUNK_BATCH_FINISHED)]
pub struct CChunkBatchEnd {
    batch_size: VarInt,
}

impl CChunkBatchEnd {
    pub fn new(count: u16) -> Self {
        Self {
            batch_size: count.into(),
        }
    }
}
