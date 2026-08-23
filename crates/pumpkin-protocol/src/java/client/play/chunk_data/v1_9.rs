use super::util::{get_light_bytes, pack_legacy_data, pack_modern_data, write_compound_nbt};
use crate::VarInt;
use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::ser::NetworkWriteExt;
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_util::encompassing_bits;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

/// Serializes chunk data for Minecraft 1.9 through 1.17.1 (including 1.12.2).
#[expect(clippy::too_many_lines)]
pub fn write_chunk_data(
    chunk: &ChunkData,
    mut write: impl Write,
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    write.write_i32_be(chunk.x)?;
    write.write_i32_be(chunk.z)?;

    if version < &JavaMinecraftVersion::V_1_17 {
        write.write_bool(true)?; // full chunk
    }
    if version == &JavaMinecraftVersion::V_1_16 || version == &JavaMinecraftVersion::V_1_16_1 {
        write.write_bool(true)?; // ignore old data
    }

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

    let mut chunk_mask = 0u32;
    for i in 0..16 {
        let section_idx = base_section + i;
        if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air() {
            chunk_mask |= 1 << i;
        }
    }

    if version >= &JavaMinecraftVersion::V_1_17 {
        write.write_bitset(&BitSet(Box::new([chunk_mask as i64])))?;
    } else {
        write.write_var_int(&VarInt(chunk_mask as i32))?;
    }

    if version >= &JavaMinecraftVersion::V_1_14 {
        let heightmaps = chunk
            .heightmap
            .lock()
            .map_err(|_| WritingError::Message("heightmap lock poisoned".into()))?;
        let mut comp = pumpkin_nbt::compound::NbtCompound::new();

        let (ws_vec, mb_vec, mbnl_vec) = if version < &JavaMinecraftVersion::V_1_16 {
            (
                convert_heightmap_to_legacy(
                    heightmaps.world_surface.as_deref(),
                    chunk.section.min_y,
                ),
                convert_heightmap_to_legacy(
                    heightmaps.motion_blocking.as_deref(),
                    chunk.section.min_y,
                ),
                convert_heightmap_to_legacy(
                    heightmaps.motion_blocking_no_leaves.as_deref(),
                    chunk.section.min_y,
                ),
            )
        } else {
            (
                heightmaps
                    .world_surface
                    .as_deref()
                    .unwrap_or(&[0; 37])
                    .to_vec(),
                heightmaps
                    .motion_blocking
                    .as_deref()
                    .unwrap_or(&[0; 37])
                    .to_vec(),
                heightmaps
                    .motion_blocking_no_leaves
                    .as_deref()
                    .unwrap_or(&[0; 37])
                    .to_vec(),
            )
        };

        comp.put("WORLD_SURFACE", pumpkin_nbt::tag::NbtTag::LongArray(ws_vec));
        comp.put(
            "MOTION_BLOCKING",
            pumpkin_nbt::tag::NbtTag::LongArray(mb_vec),
        );
        comp.put(
            "MOTION_BLOCKING_NO_LEAVES",
            pumpkin_nbt::tag::NbtTag::LongArray(mbnl_vec),
        );
        write_compound_nbt(&mut write, comp, *version)?;
    }

    if version >= &JavaMinecraftVersion::V_1_15 {
        if version >= &JavaMinecraftVersion::V_1_16_2 {
            write.write_var_int(&VarInt(1024))?;
        }
        for i in 0..16 {
            let section_idx = base_section + i;
            let biome_section = if section_idx < biome_sections.len() {
                &biome_sections[section_idx]
            } else {
                &biome_sections[0]
            };
            for y in 0..4 {
                for z in 0..4 {
                    for x in 0..4 {
                        let biome_id = biome_section.get(x, y, z);
                        if version >= &JavaMinecraftVersion::V_1_16_2 {
                            write.write_var_int(&VarInt(i32::from(biome_id)))?;
                        } else {
                            write.write_i32_be(i32::from(biome_id))?;
                        }
                    }
                }
            }
        }
    }

    let mut data_buf = Vec::new();
    for i in 0..16 {
        if (chunk_mask & (1 << i)) != 0 {
            let section_idx = base_section + i;
            let section = &block_sections[section_idx];
            if version >= &JavaMinecraftVersion::V_1_14 {
                data_buf.write_i16_be(section.non_air_block_count() as i16)?;
            }

            let mut state_ids = Vec::with_capacity(4096);
            let mut unique_states = Vec::new();
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let raw_state = section.get(x, y, z);
                        let remapped = remap_block_state_for_version(raw_state.as_u16(), *version);
                        state_ids.push(remapped);
                        if !unique_states.contains(&remapped) {
                            unique_states.push(remapped);
                        }
                    }
                }
            }

            if unique_states.len() <= 1 {
                let single_id = unique_states.first().copied().unwrap_or(0);
                data_buf.write_u8(4)?;
                data_buf.write_var_int(&VarInt(1))?;
                data_buf.write_var_int(&VarInt(i32::from(single_id)))?;
                let zeros = vec![0i64; 256];
                data_buf.write_list(&zeros, |buf, &packed| buf.write_i64_be(packed))?;
            } else if unique_states.len() <= 256 {
                let bits_per_entry = (encompassing_bits(unique_states.len()) as usize).max(4);
                if bits_per_entry <= 8 {
                    data_buf.write_u8(bits_per_entry as u8)?;
                    data_buf.write_var_int(&VarInt(unique_states.len() as i32))?;
                    for state in &unique_states {
                        data_buf.write_var_int(&VarInt(i32::from(*state)))?;
                    }
                    let indices: Vec<u32> = state_ids
                        .iter()
                        .map(|s| unique_states.iter().position(|u| u == s).unwrap_or(0) as u32)
                        .collect();
                    let packed = if version >= &JavaMinecraftVersion::V_1_16 {
                        pack_modern_data(&indices, bits_per_entry)
                    } else {
                        pack_legacy_data(&indices, bits_per_entry)
                    };
                    data_buf.write_list(&packed, |buf, &p| buf.write_i64_be(p))?;
                } else {
                    let direct_bpe = if version >= &JavaMinecraftVersion::V_1_16 {
                        15
                    } else if version >= &JavaMinecraftVersion::V_1_13 {
                        14
                    } else {
                        13
                    };
                    data_buf.write_u8(direct_bpe as u8)?;
                    let direct_indices: Vec<u32> =
                        state_ids.iter().map(|&s| u32::from(s)).collect();
                    let packed = if version >= &JavaMinecraftVersion::V_1_16 {
                        pack_modern_data(&direct_indices, direct_bpe)
                    } else {
                        pack_legacy_data(&direct_indices, direct_bpe)
                    };
                    data_buf.write_list(&packed, |buf, &p| buf.write_i64_be(p))?;
                }
            } else {
                let direct_bpe = if version >= &JavaMinecraftVersion::V_1_16 {
                    15
                } else if version >= &JavaMinecraftVersion::V_1_13 {
                    14
                } else {
                    13
                };
                data_buf.write_u8(direct_bpe as u8)?;
                let direct_indices: Vec<u32> = state_ids.iter().map(|&s| u32::from(s)).collect();
                let packed = if version >= &JavaMinecraftVersion::V_1_16 {
                    pack_modern_data(&direct_indices, direct_bpe)
                } else {
                    pack_legacy_data(&direct_indices, direct_bpe)
                };
                data_buf.write_list(&packed, |buf, &p| buf.write_i64_be(p))?;
            }

            if version < &JavaMinecraftVersion::V_1_14 {
                let block_light = get_light_bytes(light_engine.block_light.get(section_idx), 0);
                data_buf.write_slice(&block_light)?;
                let sky_light = get_light_bytes(light_engine.sky_light.get(section_idx), 15);
                data_buf.write_slice(&sky_light)?;
            }
        }
    }

    if version < &JavaMinecraftVersion::V_1_15 {
        for z in 0..16 {
            for x in 0..16 {
                let biome_id = if base_section < biome_sections.len() {
                    biome_sections[base_section].get(x / 4, 0, z / 4)
                } else {
                    0
                };
                if version >= &JavaMinecraftVersion::V_1_13 {
                    data_buf.write_i32_be(i32::from(biome_id))?;
                } else {
                    data_buf.write_u8(biome_id)?;
                }
            }
        }
    }

    write.write_var_int(&VarInt(data_buf.len() as i32))?;
    write.write_slice(&data_buf)?;

    if version >= &JavaMinecraftVersion::V_1_9_3 {
        let block_entities = chunk
            .pending_block_entities
            .lock()
            .map_err(|_| WritingError::Message("block_entities lock poisoned".into()))?;
        let valid_entities: Vec<_> = block_entities
            .iter()
            .filter(|(pos, _)| pos.0.y >= 0 && pos.0.y < 256)
            .collect();
        write.write_var_int(&VarInt(valid_entities.len() as i32))?;
        for (pos, nbt) in valid_entities {
            let mut entity_nbt = nbt.clone();
            entity_nbt.put("x", pumpkin_nbt::tag::NbtTag::Int(pos.0.x));
            entity_nbt.put("y", pumpkin_nbt::tag::NbtTag::Int(pos.0.y));
            entity_nbt.put("z", pumpkin_nbt::tag::NbtTag::Int(pos.0.z));
            write_compound_nbt(&mut write, entity_nbt, *version)?;
        }
    }

    Ok(())
}

fn convert_heightmap_to_legacy(modern_heightmap: Option<&[i64]>, min_y: i32) -> Vec<i64> {
    let mut heights = vec![0u32; 256];
    if let Some(modern_data) = modern_heightmap {
        for local_z in 0..16 {
            for local_x in 0..16 {
                let column_idx = local_z * 16 + local_x;
                let array_idx = column_idx / 7;
                let shift = (column_idx % 7) * 9;
                let raw_val = if array_idx < modern_data.len() {
                    ((modern_data[array_idx] as u64) >> shift) & 0x1FF
                } else {
                    0
                };
                if raw_val > 0 {
                    let world_y = (raw_val as i32) + min_y - 1;
                    let legacy_val = (world_y + 1).clamp(0, 256) as u32;
                    heights[column_idx] = legacy_val;
                }
            }
        }
    }
    pack_legacy_data(&heights, 9)
}
