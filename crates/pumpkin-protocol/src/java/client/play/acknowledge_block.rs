use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_BLOCK_CHANGED_ACK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
/// Sent by the server to acknowledge a sequence of block changes initiated by the client.
///
/// This packet is critical for preventing "ghost blocks" and synchronization issues.
/// It tells the client that the server has processed all actions up to a specific point.
#[java_packet(PLAY_BLOCK_CHANGED_ACK)]
pub struct CAcknowledgeBlockChange {
    /// The ID of the last sequence processed by the server.
    ///
    /// The client increments this ID every time it starts a sequence of actions
    /// (like breaking or placing a block), and the server must mirror it back
    /// to confirm processing is complete.
    pub sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    #[must_use]
    pub const fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}

impl ClientPacket for CAcknowledgeBlockChange {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.sequence_id)?;
        Ok(())
    }
}
