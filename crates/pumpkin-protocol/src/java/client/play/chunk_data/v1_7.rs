use super::util::get_light_bytes;
use crate::WritingError;
use crate::ser::NetworkWriteExt;
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

/// Serializes chunk data for Minecraft 1.7.2 / 1.7.6 / 1.7.10.
#[expect(clippy::too_many_lines)]
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
    let mut extended_chunk_mask = 0u16;
    for i in 0..16 {
        let section_idx = base_section + i;
        if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air() {
            chunk_mask |= 1 << i;
            let section = &block_sections[section_idx];
            let mut has_extended = false;
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let state_id = section.get(x, y, z);
                        let remapped = remap_block_state_for_version(state_id.as_u16(), *version);
                        if (remapped >> 4) > 255 {
                            has_extended = true;
                            break;
                        }
                    }
                    if has_extended {
                        break;
                    }
                }
                if has_extended {
                    break;
                }
            }
            if has_extended {
                extended_chunk_mask |= 1 << i;
            }
        }
    }
    write.write_u16_be(chunk_mask)?;
    write.write_u16_be(extended_chunk_mask)?;

    let mut raw_buf = Vec::new();
    // Pass 1: Block IDs (4096 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let section = &block_sections[section_idx];
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let state_id = section.get(x, y, z);
                        let remapped = remap_block_state_for_version(state_id.as_u16(), *version);
                        raw_buf.write_u8(((remapped >> 4) & 0xFF) as u8)?;
                    }
                }
            }
        }
    }

    // Pass 2: Metadata (2048 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let section = &block_sections[section_idx];
            for y in 0..16 {
                for z in 0..16 {
                    for x in (0..16).step_by(2) {
                        let state0 = section.get(x, y, z);
                        let remap0 = remap_block_state_for_version(state0.as_u16(), *version);
                        let state1 = section.get(x + 1, y, z);
                        let remap1 = remap_block_state_for_version(state1.as_u16(), *version);
                        let meta0 = (remap0 & 0x0F) as u8;
                        let meta1 = (remap1 & 0x0F) as u8;
                        raw_buf.write_u8(meta0 | (meta1 << 4))?;
                    }
                }
            }
        }
    }

    // Pass 3: Block light (2048 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let block_light = get_light_bytes(light_engine.block_light.get(section_idx), 0);
            raw_buf.write_slice(&block_light)?;
        }
    }

    // Pass 4: Sky light (2048 bytes per active section)
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let sky_light = get_light_bytes(light_engine.sky_light.get(section_idx), 15);
            raw_buf.write_slice(&sky_light)?;
        }
    }

    // Pass 5: Extended block data (2048 bytes per section in extended_chunk_mask)
    if extended_chunk_mask != 0 {
        for i in 0..16 {
            if (extended_chunk_mask & (1 << i)) != 0 {
                let section_idx = base_section + i;
                let section = &block_sections[section_idx];
                for y in 0..16 {
                    for z in 0..16 {
                        for x in (0..16).step_by(2) {
                            let state0 = section.get(x, y, z);
                            let remap0 = remap_block_state_for_version(state0.as_u16(), *version);
                            let state1 = section.get(x + 1, y, z);
                            let remap1 = remap_block_state_for_version(state1.as_u16(), *version);
                            let ext0 = (((remap0 >> 4) >> 8) & 0x0F) as u8;
                            let ext1 = (((remap1 >> 4) >> 8) & 0x0F) as u8;
                            raw_buf.write_u8(ext0 | (ext1 << 4))?;
                        }
                    }
                }
            }
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
            raw_buf.write_u8(biome_id)?;
        }
    }

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw_buf)?;
    let compressed = encoder.finish()?;
    write.write_i32_be(compressed.len() as i32)?;
    write.write_slice(&compressed)?;
    Ok(())
}
