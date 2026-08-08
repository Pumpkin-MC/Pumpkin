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
pub struct CLightUpdate<'a>(pub &'a ChunkData, pub Option<&'a [usize]>);

impl<'a> CLightUpdate<'a> {
    #[must_use]
    pub const fn new(chunk: &'a ChunkData) -> Self {
        Self(chunk, None)
    }

    #[must_use]
    pub const fn sections(chunk: &'a ChunkData, sections: &'a [usize]) -> Self {
        Self(chunk, Some(sections))
    }
}

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

/// Java's `DataLayer.isEmpty()` is true only for an implicit zero-filled layer.
/// `Empty(15)` is a uniform layer in Pumpkin's storage representation, but it is
/// a real full-data layer in the wire protocol and must be included in the data
/// mask with a serialized 0xFF array.
pub(super) const fn light_container_has_data(container: &LightContainer) -> bool {
    !matches!(container, LightContainer::Empty(0))
}

pub(super) fn write_light_container(
    write: &mut impl Write,
    container: &LightContainer,
) -> Result<(), WritingError> {
    let light_data_size = VarInt(LightContainer::ARRAY_SIZE as i32);
    write.write_var_int(&light_data_size)?;
    match container {
        LightContainer::Full(data) => write.write_slice(data.as_ref())?,
        LightContainer::Empty(default) => {
            let byte = default << 4 | default;
            write.write_slice(&[byte; LightContainer::ARRAY_SIZE])?;
        }
    }
    Ok(())
}

pub(super) fn light_masks(light_engine: &ChunkLight) -> LightMasks {
    light_masks_for_sections(light_engine, None)
}

pub(super) fn light_masks_for_sections(
    light_engine: &ChunkLight,
    changed_sections: Option<&[usize]>,
) -> LightMasks {
    let num_sections = light_engine.sky_light.len();
    let include_padding = changed_sections.is_none();
    let mut masks = LightMasks {
        sky: 0,
        block: 0,
        empty_sky: u64::from(include_padding),
        empty_block: u64::from(include_padding),
    };

    for section_index in 0..num_sections {
        if let Some(changed_sections) = changed_sections
            && !changed_sections.contains(&section_index)
        {
            continue;
        }

        let bit_index = section_index + 1;
        if light_container_has_data(&light_engine.sky_light[section_index]) {
            masks.sky |= 1 << bit_index;
        } else {
            masks.empty_sky |= 1 << bit_index;
        }

        if light_container_has_data(&light_engine.block_light[section_index]) {
            masks.block |= 1 << bit_index;
        } else {
            masks.empty_block |= 1 << bit_index;
        }
    }

    if changed_sections.is_none() {
        let above_world_bit = num_sections + 1;
        masks.empty_sky |= 1 << above_world_bit;
        masks.empty_block |= 1 << above_world_bit;
    }
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
        let masks = light_masks_for_sections(&light_engine, self.1);

        // Write Sky Light Mask
        write.write_bitset(&BitSet(Box::new([masks.sky as i64])))?;
        // Write Block Light Mask
        write.write_bitset(&BitSet(Box::new([masks.block as i64])))?;
        // Write Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([masks.empty_sky as i64])))?;
        // Write Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([masks.empty_block as i64])))?;

        // Write Sky Light arrays
        write.write_var_int(&VarInt(masks.sky.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if self
                .1
                .is_none_or(|sections| sections.contains(&section_index))
                && light_container_has_data(&light_engine.sky_light[section_index])
            {
                write_light_container(&mut write, &light_engine.sky_light[section_index])?;
            }
        }

        // Write Block Light arrays
        write.write_var_int(&VarInt(masks.block.count_ones() as i32))?;
        for section_index in 0..num_sections {
            if self
                .1
                .is_none_or(|sections| sections.contains(&section_index))
                && light_container_has_data(&light_engine.block_light[section_index])
            {
                write_light_container(&mut write, &light_engine.block_light[section_index])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_world::chunk::format::LightContainer;

    use super::{CLightUpdate, light_masks};
    use crate::ClientPacket;
    use pumpkin_data::chunk::ChunkStatus;
    use pumpkin_util::version::JavaMinecraftVersion;
    use pumpkin_world::chunk::{ChunkData, ChunkLight, ChunkSections};
    use pumpkin_world::tick::scheduler::ChunkTickScheduler;

    /// Builds a `ChunkData` whose light arrays are exactly `sky`/`block`.
    fn chunk_with_light(sky: Vec<LightContainer>, block: Vec<LightContainer>) -> ChunkData {
        let sections = sky.len();
        assert_eq!(sections, block.len());
        ChunkData {
            section: ChunkSections::new(sections, 0),
            heightmap: std::sync::Mutex::default(),
            x: 0,
            z: 0,
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks: ChunkTickScheduler::default(),
            pending_block_entities: std::sync::Mutex::default(),
            light_engine: std::sync::Mutex::new(ChunkLight {
                sky_light: sky.into_boxed_slice(),
                block_light: block.into_boxed_slice(),
            }),
            light_populated: std::sync::atomic::AtomicBool::new(true),
            status: ChunkStatus::Full,
            blending_data: None,
            unknown_nbt: pumpkin_nbt::compound::NbtCompound::new(),
            dirty: std::sync::atomic::AtomicBool::new(false),
            inhabited_time: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn serialize(chunk: &ChunkData) -> Vec<u8> {
        let mut out = Vec::new();
        CLightUpdate::new(chunk)
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        out
    }

    fn serialize_sections(chunk: &ChunkData, sections: &[usize]) -> Vec<u8> {
        let mut out = Vec::new();
        CLightUpdate::sections(chunk, sections)
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        out
    }

    /// The client's nibble layout, per the Mojang-named 1.21.4 source
    /// (`net/minecraft/world/level/chunk/DataLayer.java`):
    ///
    /// ```java
    /// getIndex(x, y, z) { return y << 8 | z << 4 | x; }
    /// getByteIndex(i)   { return i >> 1; }
    /// getNibbleIndex(i) { return i & 1; }
    /// get(i) -> data[getByteIndex(i)] >> 4 * getNibbleIndex(i) & 15
    /// ```
    ///
    /// So `y` is slowest-varying and `x` fastest, index parity 0 is the LOW nibble, and each
    /// section is `4096 / 2 = 2048` bytes (`DataLayer.SIZE`). The packet framing is from
    /// <https://minecraft.wiki/w/Java_Edition_protocol> for protocol 776 (Minecraft 26.2):
    /// four `BitSet`s (sky, block, empty sky, empty block), then a prefixed array of sky light
    /// arrays - "There is 1 array for each bit set to true in the sky light mask, starting
    /// with the lowest value" - then the same for block light, each inner array length 2048.
    ///
    /// This asserts the byte at the exact protocol offset rather than reading the code, which
    /// is what the tracker demands. Point chosen with x != z != y so that BOTH a transposed
    /// index formula and a swapped nibble parity move the answer:
    ///   correct:    idx = 2*256 + 3*16 + 1 = 561 -> byte 280, HIGH nibble  -> 0xA0
    ///   transposed: idx = 1*256 + 3*16 + 2 = 306 -> byte 153, LOW nibble
    #[test]
    fn sky_light_nibble_lands_at_the_offset_the_client_reads() {
        let mut lit = LightContainer::new_filled(0);
        lit.set(1, 2, 3, 0xA);
        let chunk = chunk_with_light(vec![lit], vec![LightContainer::new_empty(0)]);
        // chunk_x (1) + chunk_z (1) + 4 BitSets of one long each (4 * (1 + 8)) = 38
        // + sky array count VarInt (1) + inner array length VarInt(2048) (2) = 41
        let sky_array_start: usize = 1 + 1 + 4 * (1 + 8) + 1 + 2;
        let bytes = serialize(&chunk);

        // VarInt(2048) is 0x80 0x10; assert the framing before trusting the offset.
        assert_eq!(
            &bytes[sky_array_start - 3..sky_array_start],
            &[1, 0x80, 0x10]
        );

        let index = 2 * 256 + 3 * 16 + 1;
        assert_eq!(index, 561);
        assert_eq!(
            bytes[sky_array_start + (index >> 1)],
            0xA0,
            "sky light nibble for (1,2,3) must be the high nibble of byte {}",
            index >> 1
        );
        // Every other byte of the section is 0: nothing leaked to a transposed offset.
        assert!(
            bytes[sky_array_start..sky_array_start + 2048]
                .iter()
                .enumerate()
                .all(|(i, &b)| (i == index >> 1) == (b != 0))
        );
    }

    /// A mask whose popcount disagrees with the number of arrays actually written
    /// desynchronises every following array, so pin the exact serialized length and the
    /// exact mask bytes for a mixed chunk.
    #[test]
    fn array_count_matches_mask_popcount_and_total_length() {
        let chunk = chunk_with_light(
            vec![
                LightContainer::new_filled(15),
                LightContainer::new_empty(0),
                LightContainer::new_filled(7),
            ],
            vec![
                LightContainer::new_empty(0),
                LightContainer::new_filled(1),
                LightContainer::new_empty(0),
            ],
        );
        // The mask bit math assumes the light arrays are exactly as long as the block
        // sections; if that invariant breaks, `section_index + 1` is wrong.
        assert_eq!(
            chunk.light_engine.lock().unwrap().sky_light.len(),
            chunk.section.section_count()
        );
        let bytes = serialize(&chunk);

        // Physical section i is bit i+1; bit 0 and bit 4 are the below/above-world padding.
        let sky_mask: u64 = (1 << 1) | (1 << 3);
        let block_mask: u64 = 1 << 2;
        assert_eq!(&bytes[2..3], &[1]);
        assert_eq!(&bytes[3..11], &sky_mask.to_be_bytes());
        assert_eq!(&bytes[12..20], &block_mask.to_be_bytes());
        let empty_sky_mask: u64 = 1 | (1 << 2) | (1 << 4);
        let empty_block_mask: u64 = 1 | (1 << 1) | (1 << 3) | (1 << 4);
        assert_eq!(&bytes[21..29], &empty_sky_mask.to_be_bytes());
        assert_eq!(&bytes[30..38], &empty_block_mask.to_be_bytes());

        assert_eq!(bytes[38], sky_mask.count_ones() as u8);
        assert_eq!(bytes[38], 2);
        let block_count_at = 38 + 1 + 2 * (2 + 2048);
        assert_eq!(bytes[block_count_at], block_mask.count_ones() as u8);
        assert_eq!(bytes.len(), block_count_at + 1 + (2 + 2048));

        // Second sky array is the 7-filled section, at the offset the client would seek to.
        let second_sky = 38 + 1 + (2 + 2048) + 2;
        assert_eq!(bytes[second_sky], 0x77);
    }

    #[test]
    fn masks_include_padding_and_offset_physical_sections() {
        let light = ChunkLight {
            sky_light: [LightContainer::new_empty(15), LightContainer::new_empty(0)].into(),
            block_light: [LightContainer::new_empty(0), LightContainer::new_filled(1)].into(),
        };

        let masks = light_masks(&light);

        assert_eq!(masks.sky, 1 << 1);
        assert_eq!(masks.block, 1 << 2);
        assert_eq!(masks.empty_sky, (1 << 0) | (1 << 2) | (1 << 3));
        assert_eq!(masks.empty_block, (1 << 0) | (1 << 1) | (1 << 3));
    }

    #[test]
    fn uniform_full_sky_layer_is_serialized_as_data() {
        let chunk = chunk_with_light(
            vec![LightContainer::new_empty(15)],
            vec![LightContainer::new_empty(0)],
        );
        let bytes = serialize(&chunk);

        // Coordinates (1, 1) and four one-long bitsets precede the arrays.
        assert_eq!(&bytes[3..11], &(1u64 << 1).to_be_bytes());
        assert_eq!(&bytes[21..29], &(1u64 | (1 << 2)).to_be_bytes());
        let array_start = 1 + 1 + 4 * (1 + 8) + 1 + 2;
        assert_eq!(bytes[array_start..array_start + 2048], [0xFF; 2048]);
    }

    #[test]
    fn incremental_update_contains_only_changed_sections() {
        let chunk = chunk_with_light(
            vec![
                LightContainer::new_filled(1),
                LightContainer::new_empty(15),
                LightContainer::new_filled(3),
            ],
            vec![
                LightContainer::new_filled(4),
                LightContainer::new_empty(0),
                LightContainer::new_filled(5),
            ],
        );
        let bytes = serialize_sections(&chunk, &[1]);

        // Only physical section 1 (wire bit 2) is present. Incremental masks
        // intentionally omit the below/above-world padding sections.
        let changed_bit = (1u64 << 2).to_be_bytes();
        assert_eq!(&bytes[3..11], &changed_bit); // sky data mask
        assert_eq!(&bytes[12..20], &[0; 8]); // block section is empty
        assert_eq!(&bytes[21..29], &[0; 8]); // no empty sky section
        assert_eq!(&bytes[30..38], &changed_bit); // empty block mask

        let sky_count_at = 38;
        assert_eq!(bytes[sky_count_at], 1);
        let block_count_at = sky_count_at + 1 + 2 + 2048;
        assert_eq!(bytes[block_count_at], 0);
    }
}
