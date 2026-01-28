use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_START;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Signals the beginning of a new batch of chunk data packets.
///
/// This packet initiates a synchronized chunk loading sequence. In modern
/// protocol versions, the server must wrap chunk data transmissions between
/// a `Start` and `End` packet to manage client-side backpressure and
/// network throughput.
#[derive(Serialize)]
#[java_packet(PLAY_CHUNK_BATCH_START)]
pub struct CChunkBatchStart;
