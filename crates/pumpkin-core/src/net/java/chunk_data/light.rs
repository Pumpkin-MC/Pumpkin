use pumpkin_protocol::codec::bit_set::BitSet;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CLightUpdate, LightData};
use pumpkin_protocol::ser::WritingError;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;

pub trait ChunkLightExt {
    fn from_chunk(chunk: &ChunkData, version: JavaMinecraftVersion) -> Result<Self, WritingError>
    where
        Self: Sized;
}

impl ChunkLightExt for CLightUpdate {
    fn from_chunk(chunk: &ChunkData, version: JavaMinecraftVersion) -> Result<Self, WritingError> {
        let light_data = light_data_from_chunk(chunk, version)?;
        Ok(Self {
            chunk_x: VarInt(chunk.x),
            chunk_z: VarInt(chunk.z),
            light_data,
        })
    }
}

/// Adds one section to the light masks, sending a uniform non-zero section as a full array.
fn push_section(
    container: Option<&LightContainer>,
    bit: usize,
    mask: &mut u64,
    empty_mask: &mut u64,
    arrays: &mut Vec<Vec<u8>>,
) {
    match container.and_then(LightContainer::nibbles) {
        Some(data) => {
            *mask |= 1 << bit;
            arrays.push(data.to_vec());
        }
        None => *empty_mask |= 1 << bit,
    }
}

pub fn light_data_from_chunk(
    chunk: &ChunkData,
    version: JavaMinecraftVersion,
) -> Result<LightData, WritingError> {
    let light_engine = chunk
        .light_engine
        .lock()
        .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

    let mut sky_light_mask = 0u64;
    let mut block_light_mask = 0u64;
    let mut sky_light_empty_mask = 0u64;
    let mut block_light_empty_mask = 0u64;
    let mut sky_light_arrays = Vec::new();
    let mut block_light_arrays = Vec::new();

    if version < JavaMinecraftVersion::V_1_18 {
        let base_section = (0 - chunk.section.min_y).max(0) as usize / 16;

        // Bit 0: Y = -1 (below world section 0), bits 1..=16: world sections (Y = 0..15),
        // bit 17: Y = 16 (above world section 15).
        for bit in 0..18 {
            let section = (base_section + bit).checked_sub(1);
            push_section(
                section.and_then(|index| light_engine.sky_light.get(index)),
                bit,
                &mut sky_light_mask,
                &mut sky_light_empty_mask,
                &mut sky_light_arrays,
            );
            push_section(
                section.and_then(|index| light_engine.block_light.get(index)),
                bit,
                &mut block_light_mask,
                &mut block_light_empty_mask,
                &mut block_light_arrays,
            );
        }
    } else {
        let num_sections = light_engine.sky_light.len();

        sky_light_empty_mask |= 1 << 0;
        block_light_empty_mask |= 1 << 0;

        for section_index in 0..num_sections {
            let bit_index = section_index + 1;
            push_section(
                light_engine.sky_light.get(section_index),
                bit_index,
                &mut sky_light_mask,
                &mut sky_light_empty_mask,
                &mut sky_light_arrays,
            );
            push_section(
                light_engine.block_light.get(section_index),
                bit_index,
                &mut block_light_mask,
                &mut block_light_empty_mask,
                &mut block_light_arrays,
            );
        }

        sky_light_empty_mask |= 1 << (num_sections + 1);
        block_light_empty_mask |= 1 << (num_sections + 1);
    }

    Ok(LightData {
        trust_edges: true,
        sky_light_mask: BitSet::from_u64(sky_light_mask),
        block_light_mask: BitSet::from_u64(block_light_mask),
        empty_sky_light_mask: BitSet::from_u64(sky_light_empty_mask),
        empty_block_light_mask: BitSet::from_u64(block_light_empty_mask),
        sky_light_arrays,
        block_light_arrays,
    })
}
