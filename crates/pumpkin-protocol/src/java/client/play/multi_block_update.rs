use pumpkin_data::packet::clientbound::play::SECTION_BLOCKS_UPDATE;
use pumpkin_data::{BlockStateId, block_state_remap::remap_block_state_for_version};
use pumpkin_util::math::{
    position::{BlockPos, chunk_section_from_pos, pack_local_chunk_section},
    vector3::{self},
};
use pumpkin_util::version::JavaMinecraftVersion;

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
#[java_packet(SECTION_BLOCKS_UPDATE)]
pub struct CMultiBlockUpdate {
    /// Chunk section position (x << 42 | z << 20 | y)
    pub chunk_section: i64,
    /// Array of `VarLongs`: (Block State ID << 12 | Relative Position)
    pub updates: Vec<VarLong>,
}

impl CMultiBlockUpdate {
    #[must_use]
    pub fn new(updates: &[(BlockPos, BlockStateId)]) -> Self {
        let first_pos = updates[0].0;

        let chunk_section_vec = chunk_section_from_pos(&first_pos);
        let chunk_section = vector3::packed_chunk_pos(&chunk_section_vec);

        let packed_updates = updates
            .iter()
            .map(|(pos, state_id)| {
                let local_pos = pack_local_chunk_section(pos) as u64;
                let packed = (u64::from(state_id.as_u16()) << 12) | (local_pos & 0xFFF);
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        if *version >= JavaMinecraftVersion::V_1_13 {
            // 1.13+ packs the chunk section position into a single i64 and each
            // record into a VarLong: (block state id << 12) | local position.
            write.write_i64_be(self.chunk_section)?;
            write.write_var_int(&VarInt(self.updates.len() as i32))?;

            for update in &self.updates {
                let (state_id, local_pos) = unpack_update(update);
                let remapped_state_id = remap_block_state_for_version(state_id, *version);
                let remapped_packed = (u64::from(remapped_state_id) << 12) | local_pos;
                write.write_var_long(&VarLong(remapped_packed as i64))?;
            }
        } else {
            // Pre-1.13 the chunk position is sent as two separate i32s.
            let chunk_x = ((self.chunk_section >> 42) & 0x3F_FFFF) as i64;
            let chunk_z = ((self.chunk_section >> 20) & 0x3F_FFFF) as i64;
            write.write_i32_be(((chunk_x << 42) >> 42) as i32)?;
            write.write_i32_be(((chunk_z << 42) >> 42) as i32)?;

            if *version >= JavaMinecraftVersion::V_1_9 {
                // 1.9 - 1.12: each record is a u16 packed position followed by
                // a VarInt block state id.
                write.write_var_int(&VarInt(self.updates.len() as i32))?;
                for update in &self.updates {
                    let (state_id, local_pos) = unpack_update(update);
                    let remapped_state_id = remap_block_state_for_version(state_id, *version);
                    let (x, z, y) = local_coords(local_pos);
                    let packed_pos = ((x & 0xF) << 12) | ((z & 0xF) << 8) | (y & 0xF);
                    write.write_u16_be(packed_pos as u16)?;
                    write.write_var_int(&VarInt(remapped_state_id as i32))?;
                }
            } else if *version >= JavaMinecraftVersion::V_1_8 {
                // 1.8: each record is horizontal position, y and a block state id.
                write.write_var_int(&VarInt(self.updates.len() as i32))?;
                for update in &self.updates {
                    let (state_id, local_pos) = unpack_update(update);
                    let remapped_state_id = remap_block_state_for_version(state_id, *version);
                    let (x, z, y) = local_coords(local_pos);
                    write.write_u8(((x & 0xF) << 4 | (z & 0xF)) as u8)?;
                    write.write_u8(y as u8)?;
                    write.write_var_int(&VarInt(remapped_state_id as i32))?;
                }
            } else {
                // 1.7.x: the record count is a short and each record carries a
                // plain block id instead of a block state id.
                let count = i16::try_from(self.updates.len())
                    .map_err(|_| WritingError::Message("Too many block updates".into()))?;
                write.write_i16_be(count)?;
                for update in &self.updates {
                    let (state_id, local_pos) = unpack_update(update);
                    let remapped_state_id = remap_block_state_for_version(state_id, *version);
                    let block_id = remapped_state_id >> 4;
                    let (x, z, y) = local_coords(local_pos);
                    write.write_u8(((x & 0xF) << 4 | (z & 0xF)) as u8)?;
                    write.write_u8(y as u8)?;
                    write.write_var_int(&VarInt(block_id as i32))?;
                }
            }
        }

        Ok(())
    }
}

/// Splits a stored packed update into its block state id and 12-bit local position.
fn unpack_update(update: &VarLong) -> (u16, u64) {
    let packed = update.0 as u64;
    ((packed >> 12) as u16, packed & 0xFFF)
}

/// Decodes the `(x, z, y)` coordinates from a 12-bit packed local position.
fn local_coords(local_pos: u64) -> (u64, u64, u64) {
    ((local_pos >> 8) & 0xF, (local_pos >> 4) & 0xF, local_pos & 0xF)
}

#[cfg(test)]
mod tests {
    use super::CMultiBlockUpdate;
    use crate::codec::var_long::VarLong;
    use crate::ClientPacket;
    use pumpkin_util::math::vector3::{self, Vector3};
    use pumpkin_util::version::JavaMinecraftVersion;

    fn sample() -> CMultiBlockUpdate {
        // state id 5 at local position (x=1, z=2, y=3), section (x=1, y=2, z=3)
        let local_pos = (1u64 << 8) | (2u64 << 4) | 3u64;
        CMultiBlockUpdate {
            chunk_section: vector3::packed_chunk_pos(&Vector3::new(1, 2, 3)),
            updates: vec![VarLong(((5u64 << 12) | local_pos) as i64)],
        }
    }

    #[test]
    fn pre_1_9_uses_horizontal_and_y_record() {
        let packet = sample();
        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_8)
            .unwrap();

        assert_eq!(&out[0..4], &1i32.to_be_bytes());
        assert_eq!(&out[4..8], &3i32.to_be_bytes());
        assert_eq!(out[8], 1, "record count should be a 1-byte VarInt");
        assert_eq!(out[9], 0x12, "horizontal position packs x << 4 | z");
        assert_eq!(out[10], 3, "y coordinate");
        assert!(out.len() >= 12);
    }

    #[test]
    fn v1_7_uses_short_count_and_plain_block_id() {
        let packet = sample();
        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_7_6)
            .unwrap();

        assert_eq!(&out[0..4], &1i32.to_be_bytes());
        assert_eq!(&out[4..8], &3i32.to_be_bytes());
        assert_eq!(&out[8..10], &1i16.to_be_bytes(), "count is a short in 1.7");
        assert_eq!(out[10], 0x12, "horizontal position packs x << 4 | z");
        assert_eq!(out[11], 3, "y coordinate");
        assert!(out.len() >= 13);
    }

    #[test]
    fn pre_1_13_uses_chunk_x_z_and_short_record() {
        let packet = sample();
        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_9)
            .unwrap();

        assert_eq!(&out[0..4], &1i32.to_be_bytes());
        assert_eq!(&out[4..8], &3i32.to_be_bytes());
        assert_eq!(out[8], 1, "record count should be a 1-byte VarInt");
        // local position packed for 1.9-1.12: (x << 12) | (z << 8) | y
        assert_eq!(&out[9..11], &4611u16.to_be_bytes());
        // the block state id VarInt follows (at least one byte, no extra data)
        assert!(out.len() >= 12);
    }

    #[test]
    fn post_1_13_uses_section_and_var_long_record() {
        let packet = sample();
        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_13)
            .unwrap();

        assert_eq!(&out[0..8], &packet.chunk_section.to_be_bytes());
        assert_eq!(out[8], 1, "record count should be a 1-byte VarInt");
        assert!(out.len() >= 10);
    }
}
