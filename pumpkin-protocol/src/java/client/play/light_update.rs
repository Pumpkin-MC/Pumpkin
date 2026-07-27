use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LIGHT_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use std::io::Write;

/// Sent by the server to update light levels (block light and sky light) for a chunk.
///
/// This packet updates lighting data for a specific chunk without sending the full chunk data.
/// It's used when block placement or removal changes the lighting in a chunk.
#[java_packet(PLAY_LIGHT_UPDATE)]
pub struct CLightUpdate<'a>(pub &'a ChunkData);

/// Computes the four light bitset masks (sky mask, block mask, empty sky mask,
/// empty block mask) for a chunk's light sections.
///
/// Vanilla's light masks span sections `minSection - 1 ..= maxSection + 1`: bit 0 is the
/// always-empty section below the world, bits `1..=num_sections` are the real chunk
/// sections, and bit `num_sections + 1` is the always-empty section above the world.
/// This must match `CChunkData`'s light section encoding exactly, since vanilla serialises
/// both packets' light data through the identical `ClientboundLightUpdatePacketData`
/// structure.
fn compute_light_masks(
    sky_light: &[LightContainer],
    block_light: &[LightContainer],
) -> (u64, u64, u64, u64) {
    let num_sections = sky_light.len();

    let mut sky_light_empty_mask = 0u64;
    let mut block_light_empty_mask = 0u64;
    let mut sky_light_mask = 0u64;
    let mut block_light_mask = 0u64;

    // Bit 0 represents the section below the world (always empty)
    sky_light_empty_mask |= 1 << 0;
    block_light_empty_mask |= 1 << 0;

    // Bits 1..=num_sections represent the actual world sections
    for section_index in 0..num_sections {
        let bit_index = section_index + 1; // Offset by 1 for the below-world section

        if let LightContainer::Full(_) = &sky_light[section_index] {
            sky_light_mask |= 1 << bit_index;
        } else {
            sky_light_empty_mask |= 1 << bit_index;
        }

        if let LightContainer::Full(_) = &block_light[section_index] {
            block_light_mask |= 1 << bit_index;
        } else {
            block_light_empty_mask |= 1 << bit_index;
        }
    }

    // Bit num_sections+1 represents the section above the world (always empty)
    sky_light_empty_mask |= 1 << (num_sections + 1);
    block_light_empty_mask |= 1 << (num_sections + 1);

    (
        sky_light_mask,
        block_light_mask,
        sky_light_empty_mask,
        block_light_empty_mask,
    )
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

        // Light masks include sections from -1 (below world) to num_sections (above world)
        // This means we need to account for 2 extra sections in the bitset
        let light_engine = self
            .0
            .light_engine
            .lock()
            .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;
        let num_sections = light_engine.sky_light.len();

        let (sky_light_mask, block_light_mask, sky_light_empty_mask, block_light_empty_mask) =
            compute_light_masks(&light_engine.sky_light, &light_engine.block_light);

        // Write Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_mask as i64])))?;
        // Write Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_mask as i64])))?;
        // Write Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_empty_mask as i64])))?;
        // Write Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_empty_mask as i64])))?;

        let light_data_size: VarInt = VarInt(LightContainer::ARRAY_SIZE as i32);

        // Write Sky Light arrays
        write.write_var_int(&VarInt(sky_light_mask.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if let LightContainer::Full(data) = &light_engine.sky_light[section_index] {
                write.write_var_int(&light_data_size)?;
                write.write_slice(data.as_ref())?;
            }
        }

        // Write Block Light arrays
        write.write_var_int(&VarInt(block_light_mask.count_ones() as i32))?;
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
    use super::compute_light_masks;
    use pumpkin_world::chunk::format::LightContainer;

    #[test]
    fn light_masks_pad_below_and_above_world_sections() {
        // 3 real sections: index 0 full, index 1 empty, index 2 full.
        let sky_light = vec![
            LightContainer::new_filled(15),
            LightContainer::new_empty(0),
            LightContainer::new_filled(7),
        ];
        let block_light = vec![
            LightContainer::new_empty(0),
            LightContainer::new_filled(3),
            LightContainer::new_empty(0),
        ];

        let (sky_mask, block_mask, sky_empty_mask, block_empty_mask) =
            compute_light_masks(&sky_light, &block_light);

        // Real sections live at bits 1..=num_sections; bit 0 (below world) and
        // bit num_sections + 1 (above world) must always be set in the empty masks,
        // matching CChunkData's encoding and vanilla's minSection-1..=maxSection+1 span.
        assert_eq!(sky_mask, (1 << 1) | (1 << 3));
        assert_eq!(sky_empty_mask, (1 << 0) | (1 << 2) | (1 << 4));

        assert_eq!(block_mask, 1 << 2);
        assert_eq!(block_empty_mask, (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4));
    }

    #[test]
    fn light_masks_always_set_below_and_above_world_bits_even_when_fully_lit() {
        let sky_light = vec![LightContainer::new_filled(15); 4];
        let block_light = vec![LightContainer::new_filled(15); 4];

        let (sky_mask, _block_mask, sky_empty_mask, block_empty_mask) =
            compute_light_masks(&sky_light, &block_light);

        // Bit 0 and bit num_sections + 1 (here, bit 5) represent the always-empty
        // below/above-world padding sections and must never be reported as lit.
        assert_eq!(sky_mask, (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4));
        assert_eq!(sky_empty_mask, (1 << 0) | (1 << 5));
        assert_eq!(block_empty_mask, (1 << 0) | (1 << 5));
    }
}
