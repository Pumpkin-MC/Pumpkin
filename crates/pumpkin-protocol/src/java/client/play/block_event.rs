use pumpkin_data::packet::clientbound::PLAY_BLOCK_EVENT;
use pumpkin_util::math::position::BlockPos;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Triggers a physical block animation or sound effect.
///
/// This is used for simple block interactions that don't necessarily change
/// NBT data, such as chests opening/closing, pistons extending, or note blocks playing.
#[java_packet(PLAY_BLOCK_EVENT)]
pub struct CBlockEvent {
    /// The coordinates where the event occurs.
    pub location: BlockPos,
    /// The ID of the action to perform. Meaning varies by block type.
    pub action_id: u8,
    /// A parameter for the action (e.g., note pitch or instrument).
    pub action_parameter: u8,
    /// The block type ID (e.g., `minecraft:chest`).
    /// Note: This is the block ID, not the state ID.
    pub block_type: VarInt,
}

impl CBlockEvent {
    #[must_use]
    pub const fn new(
        location: BlockPos,
        action_id: u8,
        action_parameter: u8,
        block_type: VarInt,
    ) -> Self {
        Self {
            location,
            action_id,
            action_parameter,
            block_type,
        }
    }
}

impl ClientPacket for CBlockEvent {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_block_pos(&self.location)?;
        write.write_u8(self.action_id)?;
        write.write_u8(self.action_parameter)?;
        write.write_var_int(&self.block_type)?;
        Ok(())
    }
}
