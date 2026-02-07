use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::packet::clientbound::PLAY_BLOCK_UPDATE;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::MinecraftVersion;

use pumpkin_macros::java_packet;
use std::io::Write;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Updates a single block state at a specific location in the world.
///
/// This is the most common way to sync world changes to the client, such as
/// when a player places a block, a fluid flows, or a redstone component toggles.
#[java_packet(PLAY_BLOCK_UPDATE)]
pub struct CBlockUpdate {
    /// The world coordinates of the block being updated.
    pub location: BlockPos,
    /// The new block state ID.
    pub state_id: VarInt,
}

impl CBlockUpdate {
    #[must_use]
    pub const fn new(location: BlockPos, state_id: VarInt) -> Self {
        Self { location, state_id }
    }
}

impl ClientPacket for CBlockUpdate {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_block_pos(&self.location)?;

        let remapped_state = match u16::try_from(self.state_id.0) {
            Ok(state_id) => i32::from(remap_block_state_for_version(state_id, *version)),
            Err(_) => self.state_id.0,
        };
        write.write_var_int(&VarInt(remapped_state))?;

        Ok(())
    }
}
