use crate::ClientPacket;
use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_START;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
/// Signals the beginning of a new batch of chunk data packets.
///
/// This packet initiates a synchronized chunk loading sequence. In modern
/// protocol versions, the server must wrap chunk data transmissions between
/// a `Start` and `End` packet to manage client-side backpressure and
/// network throughput.
#[java_packet(PLAY_CHUNK_BATCH_START)]
pub struct CChunkBatchStart;

impl ClientPacket for CChunkBatchStart {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
