use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LIGHT_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::{ChunkData, ChunkLight};
use std::io::Write;

/// Sent by the server to update light levels (block light and sky light) for a chunk.
///
/// This packet updates lighting data for a specific chunk without sending the full chunk data.
/// It's used when block placement or removal changes the lighting in a chunk.
#[java_packet(PLAY_LIGHT_UPDATE)]
pub struct CLightUpdate<'a>(pub &'a ChunkData);

/// The four masks shared by initial chunk data and incremental light updates.
///
/// Minecraft numbers light sections from the padding section below the world, so physical chunk
/// section zero is bit one and the final above-world padding section is also explicitly empty.
pub(super) struct LightMasks {
    pub sky: u64,
    pub block: u64,
    pub empty_sky: u64,
    pub empty_block: u64,
}

pub(super) fn light_masks(light_engine: &ChunkLight) -> LightMasks {
    let num_sections = light_engine.sky_light.len();
    let mut masks = LightMasks {
        sky: 0,
        block: 0,
        empty_sky: 1,
        empty_block: 1,
    };

    for section_index in 0..num_sections {
        let bit_index = section_index + 1;
        if matches!(
            light_engine.sky_light[section_index],
            LightContainer::Full(_)
        ) {
            masks.sky |= 1 << bit_index;
        } else {
            masks.empty_sky |= 1 << bit_index;
        }

        if matches!(
            light_engine.block_light[section_index],
            LightContainer::Full(_)
        ) {
            masks.block |= 1 << bit_index;
        } else {
            masks.empty_block |= 1 << bit_index;
        }
    }

    let above_world_bit = num_sections + 1;
    masks.empty_sky |= 1 << above_world_bit;
    masks.empty_block |= 1 << above_world_bit;
    masks
}

impl ClientPacket for CLightUpdate<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_var_int(&VarInt(self.0.x))?;
        // Chunk Z
        write.write_var_int(&VarInt(self.0.z))?;

        let light_engine = self
            .0
            .light_engine
            .lock()
            .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;
        let num_sections = light_engine.sky_light.len();
        let masks = light_masks(&light_engine);

        // Write Sky Light Mask
        write.write_bitset(&BitSet(Box::new([masks.sky as i64])))?;
        // Write Block Light Mask
        write.write_bitset(&BitSet(Box::new([masks.block as i64])))?;
        // Write Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([masks.empty_sky as i64])))?;
        // Write Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([masks.empty_block as i64])))?;

        let light_data_size: VarInt = VarInt(LightContainer::ARRAY_SIZE as i32);

        // Write Sky Light arrays
        write.write_var_int(&VarInt(masks.sky.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if let LightContainer::Full(data) = &light_engine.sky_light[section_index] {
                write.write_var_int(&light_data_size)?;
                write.write_slice(data.as_ref())?;
            }
        }

        // Write Block Light arrays
        write.write_var_int(&VarInt(masks.block.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if let LightContainer::Full(data) = &light_engine.block_light[section_index] {
                write.write_var_int(&light_data_size)?;
                write.write_slice(data.as_ref())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_world::chunk::format::LightContainer;

    use super::light_masks;
    use pumpkin_world::chunk::ChunkLight;

    #[test]
    fn masks_include_padding_and_offset_physical_sections() {
        let light = ChunkLight {
            sky_light: [LightContainer::new_filled(15), LightContainer::new_empty(0)].into(),
            block_light: [LightContainer::new_empty(0), LightContainer::new_filled(1)].into(),
        };

        let masks = light_masks(&light);

        assert_eq!(masks.sky, 1 << 1);
        assert_eq!(masks.block, 1 << 2);
        assert_eq!(masks.empty_sky, (1 << 0) | (1 << 2) | (1 << 3));
        assert_eq!(masks.empty_block, (1 << 0) | (1 << 1) | (1 << 3));
    }
}
