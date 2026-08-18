use pumpkin_data::packet::clientbound::PLAY_BLOCK_DESTRUCTION;
use pumpkin_util::math::position::BlockPos;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Updates the visual "breaking" progress of a block for all clients.
///
/// This packet controls the overlay of cracks that appear on a block when
/// it is being mined. It is often used to show other players' mining progress.
#[java_packet(PLAY_BLOCK_DESTRUCTION)]
pub struct CSetBlockDestroyStage {
    /// A unique ID for this destruction instance. Usually the miner's Entity ID.
    /// If multiple entities mine the same block, they must use different IDs.
    pub entity_id: VarInt,
    /// The coordinates of the block being destroyed.
    pub location: BlockPos,
    /// The destruction stage, typically a value from 0 to 9.
    /// Any value outside 0-9 (like -1) will remove the destruction overlay.
    pub destroy_stage: i8,
}

impl CSetBlockDestroyStage {
    #[must_use]
    pub const fn new(entity_id: VarInt, location: BlockPos, destroy_stage: i8) -> Self {
        Self {
            entity_id,
            location,
            destroy_stage,
        }
    }
}

impl ClientPacket for CSetBlockDestroyStage {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_block_pos(&self.location)?;
        write.write_i8(self.destroy_stage)?;
        Ok(())
    }
}
