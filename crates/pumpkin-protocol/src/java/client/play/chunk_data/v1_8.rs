use super::util::get_light_bytes;
use crate::VarInt;
use crate::WritingError;
use crate::ser::NetworkWriteExt;
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

/// Serializes chunk data for Minecraft 1.8.x.
pub fn write_chunk_data(
    chunk: &ChunkData,
    mut write: impl Write,
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    write.write_i32_be(chunk.x)?;
    write.write_i32_be(chunk.z)?;
    write.write_bool(true)?; // full chunk

    let block_sections = chunk
        .section
        .block_sections
        .read()
        .map_err(|_| WritingError::Message("block_sections read lock poisoned".into()))?;
    let biome_sections = chunk
        .section
        .biome_sections
        .read()
        .map_err(|_| WritingError::Message("biome_sections read lock poisoned".into()))?;
    let light_engine = chunk
        .light_engine
        .lock()
        .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

    let base_section = (0 - chunk.section.min_y).max(0) as usize / 16;

    let mut chunk_mask = 0u16;
    for i in 0..16 {
        let section_idx = base_section + i;
        if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air() {
            chunk_mask |= 1 << i;
        }
    }
    write.write_u16_be(chunk_mask)?;

    let mut data_buf = Vec::new();
    // Pass 1: Blocks (4096 u16 per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let section = &block_sections[section_idx];
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let state_id = section.get(x, y, z);
                        let remapped = remap_block_state_for_version(state_id.as_u16(), *version);
                        data_buf.write_all(&remapped.to_le_bytes())?;
                    }
                }
            }
        }
    }

    // Pass 2: Block light (2048 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let block_light = get_light_bytes(light_engine.block_light.get(section_idx), 0);
            data_buf.write_slice(&block_light)?;
        }
    }

    // Pass 3: Sky light (2048 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let sky_light = get_light_bytes(light_engine.sky_light.get(section_idx), 15);
            data_buf.write_slice(&sky_light)?;
        }
    }

    // Biomes (256 bytes)
    for z in 0..16 {
        for x in 0..16 {
            let biome_id = if base_section < biome_sections.len() {
                biome_sections[base_section].get(x / 4, 0, z / 4)
            } else {
                0
            };
            data_buf.write_u8(biome_id)?;
        }
    }

    write.write_var_int(&VarInt(data_buf.len() as i32))?;
    write.write_slice(&data_buf)?;
    Ok(())
}
