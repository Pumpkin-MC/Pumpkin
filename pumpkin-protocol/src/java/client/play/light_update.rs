use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LIGHT_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

/// Sent by the server to update light levels (block light and sky light) for a chunk.
///
/// This packet updates lighting data for a specific chunk without sending the full chunk data.
/// It's used when block placement or removal changes the lighting in a chunk.
#[java_packet(PLAY_LIGHT_UPDATE)]
pub struct CLightUpdate<'a>(pub &'a ChunkData);

impl ClientPacket for CLightUpdate<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_var_int(&VarInt(self.0.x))?;
        // Chunk Z
        write.write_var_int(&VarInt(self.0.z))?;

        // Light masks include sections from -1 (below world) to num_sections (above world)
        // This means we need to account for 2 extra sections in the bitset
        let light_engine = self.0.light_engine.lock().unwrap();
        let num_sections = light_engine.sky_light.len();

        let mut sky_light_empty_mask = 0u64;
        let mut block_light_empty_mask = 0u64;
        let mut sky_light_mask = 0u64;
        let mut block_light_mask = 0u64;

        // Bits 0..num_sections represent the sections including padding
        for section_index in 0..num_sections {
            let bit_index = section_index;

            if let LightContainer::Full(_) = &light_engine.sky_light[section_index] {
                sky_light_mask |= 1 << bit_index;
            } else {
                sky_light_empty_mask |= 1 << bit_index;
            }

            if let LightContainer::Full(_) = &light_engine.block_light[section_index] {
                block_light_mask |= 1 << bit_index;
            } else {
                block_light_empty_mask |= 1 << bit_index;
            }
        }

        // Write Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_mask.try_into().unwrap()])))?;
        // Write Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_mask.try_into().unwrap()])))?;
        // Write Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_empty_mask
            .try_into()
            .unwrap()])))?;
        // Write Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_empty_mask
            .try_into()
            .unwrap()])))?;

        let light_data_size: VarInt = LightContainer::ARRAY_SIZE.try_into().unwrap();

        // Write Sky Light arrays
        write.write_var_int(&VarInt(sky_light_mask.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if let LightContainer::Full(data) = &light_engine.sky_light[section_index] {
                // Ensure network nibble ordering matches client expectations
                // by swapping high/low nibbles per byte.
                write.write_var_int(&light_data_size)?;
                let mut swapped = Vec::with_capacity(data.len());
                for &b in data.iter() {
                    swapped.push(b.rotate_right(4));
                }
                write.write_slice(&swapped)?;
            }
        }

        // Write Block Light arrays
        write.write_var_int(&VarInt(block_light_mask.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if let LightContainer::Full(data) = &light_engine.block_light[section_index] {
                write.write_var_int(&light_data_size)?;
                let mut swapped = Vec::with_capacity(data.len());
                for &b in data.iter() {
                    swapped.push(b.rotate_right(4));
                }
                write.write_slice(&swapped)?;
            }
        }

        Ok(())
    }
}
