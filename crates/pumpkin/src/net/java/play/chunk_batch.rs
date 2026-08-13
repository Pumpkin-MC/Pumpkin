#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_chunk_batch(&self, player: &Player, packet: SChunkBatch) {
        player
            .chunk_manager
            .lock()
            .await
            .handle_acknowledge(packet.chunks_per_tick);
        trace!(
            "Client requested {} chunks per tick",
            packet.chunks_per_tick
        );
    }
}
