use pumpkin_data::BlockStateId;
use pumpkin_data::packet::clientbound::play::SECTION_BLOCKS_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::{BlockPos, chunk_section_from_pos, pack_local_chunk_section};
use pumpkin_util::math::vector3::{self, Vector3};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

use crate::{
    ClientPacket, ServerPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

/// Updates multiple blocks within a single chunk section (or chunk in older versions).
///
/// This packet is much more efficient than sending multiple individual
/// `CBlockUpdate` packets when many changes occur in the same area
/// (e.g., explosions, structure generation, or large-scale terraforming).
#[java_packet(SECTION_BLOCKS_UPDATE)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CMultiBlockUpdate {
    /// Chunk section position (x, y, z)
    pub chunk_section: Vector3<i32>,
    /// Suppress light updates (used in 1.16..=1.19.4)
    pub suppress_light_updates: bool,
    /// Array of block updates: (`BlockPos`, `BlockStateId`)
    pub updates: Vec<(BlockPos, BlockStateId)>,
}

impl CMultiBlockUpdate {
    #[must_use]
    pub fn new(updates: &[(BlockPos, BlockStateId)]) -> Self {
        let chunk_section = updates
            .first()
            .map_or_else(Vector3::default, |(pos, _)| chunk_section_from_pos(pos));

        Self {
            chunk_section,
            suppress_light_updates: false,
            updates: updates.to_vec(),
        }
    }

    #[must_use]
    pub const fn new_with_section(
        chunk_section: Vector3<i32>,
        suppress_light_updates: bool,
        updates: Vec<(BlockPos, BlockStateId)>,
    ) -> Self {
        Self {
            chunk_section,
            suppress_light_updates,
            updates,
        }
    }
}

impl ClientPacket for CMultiBlockUpdate {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        {
            let chunk_section = vector3::packed_chunk_pos(&self.chunk_section);
            write.write_i64_be(chunk_section)?;

            write.write_var_int(&VarInt(self.updates.len() as i32))?;

            for (pos, state_id) in &self.updates {
                let local_pos = pack_local_chunk_section(pos) as u64;
                let raw_state_id = state_id.as_u16();
                let packed = (u64::from(raw_state_id) << 12) | (local_pos & 0xFFF);
                write.write_var_long(&VarLong(packed as i64))?;
            }
        }

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CMultiBlockUpdate {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        {
            let encoded_pos = bytebuf.get_i64_be()?;
            let chunk_section = vector3::unpacked_chunk_pos(encoded_pos);

            let suppress_light_updates = false;

            let count = bytebuf.get_var_int()?.0 as usize;
            let mut updates = Vec::with_capacity(count);
            for _ in 0..count {
                let val = bytebuf.get_var_long()?.0 as u64;
                let local_pos = (val & 0xFFF) as i32;
                let state_id = (val >> 12) as u16;

                let rel_x = (local_pos >> 8) & 0xF;
                let rel_z = (local_pos >> 4) & 0xF;
                let rel_y = local_pos & 0xF;

                let block_pos = BlockPos::new(
                    (chunk_section.x << 4) + rel_x,
                    (chunk_section.y << 4) + rel_y,
                    (chunk_section.z << 4) + rel_z,
                );
                updates.push((block_pos, BlockStateId::new_or_air(state_id)));
            }

            Ok(Self {
                chunk_section,
                suppress_light_updates,
                updates,
            })
        }
    }
}
