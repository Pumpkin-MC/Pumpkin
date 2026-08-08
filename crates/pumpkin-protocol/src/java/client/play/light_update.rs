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

        write.write_var_int(&VarInt(self.0.x))?;
        write.write_var_int(&VarInt(self.0.z))?;

        let light_engine = self
            .0
            .light_engine
            .lock()
            .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;
        let num_sections = light_engine.sky_light.len();
        let masks = light_masks_for_sections(&light_engine, self.1);

        write.write_bitset(&BitSet(Box::new([masks.sky as i64])))?;
        write.write_bitset(&BitSet(Box::new([masks.block as i64])))?;
        write.write_bitset(&BitSet(Box::new([masks.empty_sky as i64])))?;
        write.write_bitset(&BitSet(Box::new([masks.empty_block as i64])))?;

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
