use pumpkin_data::packet::clientbound::play::SECTION_BLOCKS_UPDATE;
use pumpkin_data::{BlockStateId, block_state_remap::remap_block_state_for_version};
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_16 {
            let chunk_section = vector3::packed_chunk_pos(&self.chunk_section);
            write.write_i64_be(chunk_section)?;

            if *version <= JavaMinecraftVersion::V_1_19_4 {
                write.write_bool(self.suppress_light_updates)?;
            }

            write.write_var_int(&VarInt(self.updates.len() as i32))?;

            for (pos, state_id) in &self.updates {
                let local_pos = pack_local_chunk_section(pos) as u64;
                let remapped_state_id = remap_block_state_for_version(state_id.as_u16(), *version);
                let packed = (u64::from(remapped_state_id) << 12) | (local_pos & 0xFFF);
                write.write_var_long(&VarLong(packed as i64))?;
            }
        } else if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.chunk_section.x)?;
            write.write_i32_be(self.chunk_section.z)?;
            write.write_i16_be(self.updates.len() as i16)?;
            write.write_i32_be((self.updates.len() * 4) as i32)?;

            for (pos, state_id) in &self.updates {
                let rel_x = (pos.0.x & 0xF) as u16;
                let rel_z = (pos.0.z & 0xF) as u16;
                let rel_y = (pos.0.y & 0xFF) as u16;
                let packed_pos = (rel_x << 12) | (rel_z << 8) | rel_y;
                write.write_i16_be(packed_pos as i16)?;

                let remapped_state_id = remap_block_state_for_version(state_id.as_u16(), *version);
                write.write_i16_be(remapped_state_id as i16)?;
            }
        } else {
            write.write_i32_be(self.chunk_section.x)?;
            write.write_i32_be(self.chunk_section.z)?;
            write.write_var_int(&VarInt(self.updates.len() as i32))?;

            for (pos, state_id) in &self.updates {
                let rel_x = (pos.0.x & 0xF) as u16;
                let rel_z = (pos.0.z & 0xF) as u16;
                let rel_y = (pos.0.y & 0xFF) as u16;
                let packed_pos = (rel_x << 12) | (rel_z << 8) | rel_y;
                write.write_i16_be(packed_pos as i16)?;

                let remapped_state_id = remap_block_state_for_version(state_id.as_u16(), *version);
                write.write_var_int(&VarInt(i32::from(remapped_state_id)))?;
            }
        }

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CMultiBlockUpdate {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_16 {
            let encoded_pos = bytebuf.get_i64_be()?;
            let chunk_section = vector3::unpacked_chunk_pos(encoded_pos);

            let suppress_light_updates = if *version <= JavaMinecraftVersion::V_1_19_4 {
                bytebuf.get_bool()?
            } else {
                false
            };

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
        } else if *version <= JavaMinecraftVersion::V_1_7_6 {
            let chunk_x = bytebuf.get_i32_be()?;
            let chunk_z = bytebuf.get_i32_be()?;
            let count = bytebuf.get_i16_be()? as usize;
            let _data_size = bytebuf.get_i32_be()?;

            let mut updates = Vec::with_capacity(count);
            for _ in 0..count {
                let pos = bytebuf.get_i16_be()? as u16;
                let rel_x = ((pos >> 12) & 0xF) as i32;
                let rel_z = ((pos >> 8) & 0xF) as i32;
                let y = (pos & 0xFF) as i32;
                let x = (chunk_x << 4) + rel_x;
                let z = (chunk_z << 4) + rel_z;

                let block_state_id = bytebuf.get_i16_be()? as u16;
                updates.push((
                    BlockPos::new(x, y, z),
                    BlockStateId::new_or_air(block_state_id),
                ));
            }

            let chunk_section = updates.first().map_or_else(
                || Vector3::new(chunk_x, 0, chunk_z),
                |(pos, _)| chunk_section_from_pos(pos),
            );

            Ok(Self {
                chunk_section,
                suppress_light_updates: false,
                updates,
            })
        } else {
            let chunk_x = bytebuf.get_i32_be()?;
            let chunk_z = bytebuf.get_i32_be()?;
            let count = bytebuf.get_var_int()?.0 as usize;

            let mut updates = Vec::with_capacity(count);
            for _ in 0..count {
                let pos = bytebuf.get_i16_be()? as u16;
                let rel_x = ((pos >> 12) & 0xF) as i32;
                let rel_z = ((pos >> 8) & 0xF) as i32;
                let y = (pos & 0xFF) as i32;
                let x = (chunk_x << 4) + rel_x;
                let z = (chunk_z << 4) + rel_z;

                let block_state_id = bytebuf.get_var_int()?.0 as u16;
                updates.push((
                    BlockPos::new(x, y, z),
                    BlockStateId::new_or_air(block_state_id),
                ));
            }

            let chunk_section = updates.first().map_or_else(
                || Vector3::new(chunk_x, 0, chunk_z),
                |(pos, _)| chunk_section_from_pos(pos),
            );

            Ok(Self {
                chunk_section,
                suppress_light_updates: false,
                updates,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CMultiBlockUpdate;
    use crate::{ClientPacket, ServerPacket};
    use pumpkin_data::BlockStateId;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::{self, Vector3};
    use pumpkin_util::version::JavaMinecraftVersion;

    fn sample() -> CMultiBlockUpdate {
        CMultiBlockUpdate::new_with_section(
            Vector3::new(1, 2, 3),
            false,
            vec![(BlockPos::new(17, 35, 50), BlockStateId::AIR)],
        )
    }

    fn encode(version: JavaMinecraftVersion) -> Vec<u8> {
        let mut out = Vec::new();
        sample().write_packet_data(&mut out, &version).unwrap();
        out
    }

    #[test]
    fn v1_7_uses_short_count_and_four_byte_records() {
        let out = encode(JavaMinecraftVersion::V_1_7_6);

        assert_eq!(&out[0..4], &1i32.to_be_bytes());
        assert_eq!(&out[4..8], &3i32.to_be_bytes());
        assert_eq!(&out[8..10], &1i16.to_be_bytes());
        assert_eq!(&out[10..14], &4i32.to_be_bytes());
        assert_eq!(&out[14..16], &0x1223u16.to_be_bytes());
        assert_eq!(out.len(), 18);
    }

    #[test]
    fn v1_8_through_v1_15_use_chunk_coordinates_and_short_positions() {
        for version in [JavaMinecraftVersion::V_1_8, JavaMinecraftVersion::V_1_13] {
            let out = encode(version);

            assert_eq!(&out[0..4], &1i32.to_be_bytes());
            assert_eq!(&out[4..8], &3i32.to_be_bytes());
            assert_eq!(out[8], 1);
            assert_eq!(&out[9..11], &0x1223u16.to_be_bytes());
        }
    }

    #[test]
    fn v1_16_introduces_packed_sections_and_temporary_light_flag() {
        let expected_section = vector3::packed_chunk_pos(&Vector3::new(1, 2, 3)).to_be_bytes();

        for version in [JavaMinecraftVersion::V_1_16, JavaMinecraftVersion::V_1_19_4] {
            let out = encode(version);
            assert_eq!(&out[0..8], &expected_section);
            assert_eq!(out[8], 0, "suppress-light-updates flag");
            assert_eq!(out[9], 1, "record count");
        }

        let out = encode(JavaMinecraftVersion::V_1_20);
        assert_eq!(&out[0..8], &expected_section);
        assert_eq!(out[8], 1, "record count follows the section directly");
    }

    #[test]
    fn representative_versions_round_trip() {
        for version in [
            JavaMinecraftVersion::V_1_7_6,
            JavaMinecraftVersion::V_1_8,
            JavaMinecraftVersion::V_1_13,
            JavaMinecraftVersion::V_1_16,
            JavaMinecraftVersion::V_1_19_4,
            JavaMinecraftVersion::V_1_20,
            JavaMinecraftVersion::V_26_2,
        ] {
            let encoded = encode(version);
            let mut input = encoded.as_slice();
            let decoded = CMultiBlockUpdate::read(&mut input, &version).unwrap();

            assert_eq!(decoded, sample());
            assert!(input.is_empty());
        }
    }
}
