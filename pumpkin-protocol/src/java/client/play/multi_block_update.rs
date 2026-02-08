use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::packet::clientbound::PLAY_SECTION_BLOCKS_UPDATE;
use pumpkin_util::math::{
    position::{BlockPos, chunk_section_from_pos, pack_local_chunk_section},
    vector3::{self},
};
use pumpkin_util::version::MinecraftVersion;

use pumpkin_macros::java_packet;
use std::io::Write;

use crate::{
    ClientPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkWriteExt, WritingError},
};

/// Updates multiple blocks within a single 16x16x16 chunk section.
///
/// This packet is much more efficient than sending multiple individual
/// `CBlockUpdate` packets when many changes occur in the same area
/// (e.g., explosions, structure generation, or large-scale terraforming).
#[java_packet(PLAY_SECTION_BLOCKS_UPDATE)]
pub struct CMultiBlockUpdate {
    /// Chunk section position (x << 42 | z << 20 | y)
    pub chunk_section: i64,
    /// Array of `VarLongs`: (Block State ID << 12 | Relative Position)
    pub updates: Vec<VarLong>,
}

impl CMultiBlockUpdate {
    #[must_use]
    pub fn new(updates: &[(BlockPos, u16)]) -> Self {
        let first_pos = updates[0].0;

        let chunk_section_vec = chunk_section_from_pos(&first_pos);
        let chunk_section = vector3::packed_chunk_pos(&chunk_section_vec);

        let packed_updates = updates
            .iter()
            .map(|(pos, state_id)| {
                let local_pos = pack_local_chunk_section(pos) as u64;
                let packed = (u64::from(*state_id) << 12) | (local_pos & 0xFFF);
                VarLong(packed as i64)
            })
            .collect();

        Self {
            chunk_section,
            updates: packed_updates,
        }
    }
}
impl ClientPacket for CMultiBlockUpdate {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_i64_be(self.chunk_section)?;
        write.write_var_int(&VarInt(self.updates.len() as i32))?;

        for update in &self.updates {
            let packed_update = update.0 as u64;
            let local_pos = packed_update & 0xFFF;
            let state_id = (packed_update >> 12) as u16;
            let remapped_state_id = remap_block_state_for_version(state_id, *version);
            let remapped_packed = (u64::from(remapped_state_id) << 12) | local_pos;
            write.write_var_long(&VarLong(remapped_packed as i64))?;
        }

        Ok(())
    }
}
