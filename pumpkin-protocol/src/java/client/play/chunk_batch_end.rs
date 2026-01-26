use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_FINISHED;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

/// Notifies the client that the server has finished sending a batch of chunks.
///
/// Introduced in modern protocol versions to optimize world loading, this packet
/// allows the client to acknowledge the receipt of a group of chunks, helping
/// the server regulate the flow of data and prevent network congestion.
#[derive(Serialize)]
#[java_packet(PLAY_CHUNK_BATCH_FINISHED)]
pub struct CChunkBatchEnd {
    /// The number of chunks sent in the batch that just finished.
    pub batch_size: VarInt,
}

impl CChunkBatchEnd {
    pub fn new(count: u16) -> Self {
        Self {
            batch_size: count.into(),
        }
    }
}
