use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::packet::MultiVersionJavaPacket;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::packet::clientbound::play::LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_util::encompassing_bits;
use pumpkin_util::math::position::get_local_cord;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};
use std::io::Write;

/// Sent by the server to provide the client with the full data for a chunk.
///
/// This includes heightmaps, the actual block and biome data (organized into sections),
/// block entities (like signs or chests), and the light level information for both
/// sky and block light.
pub struct CChunkData<'a>(pub &'a ChunkData);

impl MultiVersionJavaPacket for CChunkData<'_> {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        LEVEL_CHUNK_WITH_LIGHT.to_id(version)
    }
}

impl<'a> CChunkData<'a> {
    #[must_use]
    pub const fn new(chunk: &'a ChunkData) -> Self {
        Self(chunk)
    }
}

fn write_compound_nbt(
    mut write: impl Write,
    comp: pumpkin_nbt::compound::NbtCompound,
    version: JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if version >= JavaMinecraftVersion::V_1_20_2 {
        let bytes = pumpkin_nbt::Nbt::from(comp).write_unnamed();
        write.write_all(&bytes)?;
    } else {
        let bytes = pumpkin_nbt::Nbt::from(comp).write();
        write.write_all(&bytes)?;
    }
    Ok(())
}

fn get_light_bytes(container: Option<&LightContainer>, default_val: u8) -> [u8; 2048] {
    let mut buf = [default_val << 4 | default_val; 2048];
    if let Some(LightContainer::Full(data)) = container
        && data.len() == 2048
    {
        buf.copy_from_slice(data);
    }
    buf
}

/// Bit-packs entries without spanning across 64-bit boundaries (Minecraft 1.16+ format).
fn pack_modern_data(entries: &[u32], bits_per_entry: usize) -> Vec<i64> {
    if bits_per_entry == 0 {
        return Vec::new();
    }
    let values_per_i64 = 64 / bits_per_entry;
    let long_count = entries.len().div_ceil(values_per_i64);
    let mut data = Vec::with_capacity(long_count);
    let mut current_idx = 0;
    while current_idx < entries.len() {
        let mut acc = 0u64;
        for i in 0..values_per_i64 {
            if current_idx + i < entries.len() {
                let value = entries[current_idx + i] as u64;
                acc |= value << (bits_per_entry * i);
            }
        }
        data.push(acc as i64);
        current_idx += values_per_i64;
    }
    data
}

/// Bit-packs entries across 64-bit boundaries (Minecraft 1.9 to 1.15.2 legacy format).
fn pack_legacy_data(entries: &[u32], bits_per_entry: usize) -> Vec<i64> {
    let bpe = bits_per_entry.max(4);
    let total_bits = entries.len() * bpe;
    let long_count = total_bits.div_ceil(64);
    let mut data = vec![0u64; long_count];
    let max_entry_value = (1u64 << bpe) - 1;

    for (index, &value) in entries.iter().enumerate() {
        let val = (value as u64) & max_entry_value;
        let bit_index = index * bpe;
        let start_index = bit_index / 64;
        let end_index = ((index + 1) * bpe - 1) / 64;
        let start_bit_sub_index = bit_index % 64;

        data[start_index] = (data[start_index] & !(max_entry_value << start_bit_sub_index))
            | (val << start_bit_sub_index);
        if start_index != end_index {
            let end_bit_sub_index = 64 - start_bit_sub_index;
            let j1 = bpe - end_bit_sub_index;
            data[end_index] = ((data[end_index] >> j1) << j1) | (val >> end_bit_sub_index);
        }
    }

    data.into_iter().map(|w| w as i64).collect()
}

impl ClientPacket for CChunkData<'_> {
    #[expect(clippy::too_many_lines)]
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version >= &JavaMinecraftVersion::V_1_18 {
            // Modern 1.18+ chunk serialization
            write.write_i32_be(self.0.x)?;
            write.write_i32_be(self.0.z)?;

            let heightmaps = self
                .0
                .heightmap
                .lock()
                .map_err(|_| WritingError::Message("heightmap lock poisoned".into()))?;
            if version >= &JavaMinecraftVersion::V_1_21_5 {
                write.write_var_int(&VarInt(3))?; // Map size

                let mut write_heightmap = |index: i32, data: &[i64]| -> Result<(), WritingError> {
                    write.write_var_int(&VarInt(index))?;
                    write.write_var_int(&VarInt(data.len() as i32))?;
                    for val in data {
                        write.write_i64_be(*val)?;
                    }
                    Ok(())
                };

                write_heightmap(1, heightmaps.world_surface.as_deref().unwrap_or(&[0; 37]))?;
                write_heightmap(4, heightmaps.motion_blocking.as_deref().unwrap_or(&[0; 37]))?;
                write_heightmap(
                    5,
                    heightmaps
                        .motion_blocking_no_leaves
                        .as_deref()
                        .unwrap_or(&[0; 37]),
                )?;
            } else {
                let mut comp = pumpkin_nbt::compound::NbtCompound::new();
                if let Some(ref ws) = heightmaps.world_surface {
                    comp.put(
                        "WORLD_SURFACE",
                        pumpkin_nbt::tag::NbtTag::LongArray(ws.to_vec()),
                    );
                }
                if let Some(ref mb) = heightmaps.motion_blocking {
                    comp.put(
                        "MOTION_BLOCKING",
                        pumpkin_nbt::tag::NbtTag::LongArray(mb.to_vec()),
                    );
                }
                if let Some(ref mbnl) = heightmaps.motion_blocking_no_leaves {
                    comp.put(
                        "MOTION_BLOCKING_NO_LEAVES",
                        pumpkin_nbt::tag::NbtTag::LongArray(mbnl.to_vec()),
                    );
                }
                write_compound_nbt(&mut write, comp, *version)?;
            }
            drop(heightmaps);

            {
                let mut blocks_and_biomes_buf = Vec::new();
                let block_sections = self.0.section.block_sections.read().map_err(|_| {
                    WritingError::Message("block_sections read lock poisoned".into())
                })?;
                let biome_sections = self.0.section.biome_sections.read().map_err(|_| {
                    WritingError::Message("biome_sections read lock poisoned".into())
                })?;

                let mut zero_bytes_count = 0;

                for (block_palette, biome_palette) in
                    block_sections.iter().zip(biome_sections.iter())
                {
                    let non_empty_block_count = block_palette.non_air_block_count() as i16;
                    blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;
                    if version >= &JavaMinecraftVersion::V_26_1 {
                        // New in 26.1, fluid count
                        let liquid_count = block_palette.liquid_block_count() as i16;
                        blocks_and_biomes_buf.write_i16_be(liquid_count)?;
                    }

                    let mut block_network = block_palette.convert_network();
                    if version < &CURRENT_MC_VERSION {
                        match &mut block_network.palette {
                            NetworkPalette::Single(registry_id) => {
                                *registry_id =
                                    remap_block_state_for_version(*registry_id, *version);
                            }
                            NetworkPalette::Indirect(palette) => {
                                for registry_id in palette.iter_mut() {
                                    *registry_id =
                                        remap_block_state_for_version(*registry_id, *version);
                                }
                            }
                            NetworkPalette::Direct => {
                                let bits_per_entry = usize::from(block_network.bits_per_entry);
                                let values_per_i64 = 64 / bits_per_entry;
                                let id_mask = (1u64 << bits_per_entry) - 1;

                                for packed_word in &mut block_network.packed_data {
                                    let mut remapped_word = 0u64;
                                    let packed_word_u64 = *packed_word as u64;
                                    for index in 0..values_per_i64 {
                                        let shift = index * bits_per_entry;
                                        let state_id =
                                            ((packed_word_u64 >> shift) & id_mask) as u16;
                                        let remapped_id =
                                            remap_block_state_for_version(state_id, *version);
                                        remapped_word |= u64::from(remapped_id) << shift;
                                    }
                                    *packed_word = remapped_word as i64;
                                }
                            }
                        }
                    }
                    blocks_and_biomes_buf.write_u8(block_network.bits_per_entry)?;

                    match block_network.palette {
                        NetworkPalette::Single(registry_id) => {
                            blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                        }
                        NetworkPalette::Indirect(palette) => {
                            blocks_and_biomes_buf.write_var_int(
                                &palette.len().try_into().map_err(|_| {
                                    WritingError::Message(format!(
                                        "{} is not representable as a VarInt!",
                                        palette.len()
                                    ))
                                })?,
                            )?;
                            for registry_id in palette {
                                blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                            }
                        }
                        NetworkPalette::Direct => {}
                    }

                    if version <= &JavaMinecraftVersion::V_1_21_4 {
                        blocks_and_biomes_buf
                            .write_list(&block_network.packed_data, |buf, &packed| {
                                buf.write_i64_be(packed)
                            })?;
                    } else {
                        for packed in &block_network.packed_data {
                            blocks_and_biomes_buf.write_i64_be(*packed)?;
                        }
                    }

                    let biome_network = biome_palette.convert_network();
                    blocks_and_biomes_buf.write_u8(biome_network.bits_per_entry)?;

                    match biome_network.palette {
                        NetworkPalette::Single(registry_id) => {
                            blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                        }
                        NetworkPalette::Indirect(palette) => {
                            blocks_and_biomes_buf.write_var_int(
                                &palette.len().try_into().map_err(|_| {
                                    WritingError::Message(format!(
                                        "{} is not representable as a VarInt!",
                                        palette.len()
                                    ))
                                })?,
                            )?;
                            for registry_id in palette {
                                blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                            }
                        }
                        NetworkPalette::Direct => {}
                    }

                    if version <= &JavaMinecraftVersion::V_1_21_4 {
                        blocks_and_biomes_buf
                            .write_list(&biome_network.packed_data, |buf, &packed| {
                                buf.write_i64_be(packed)
                            })?;
                    } else {
                        for packed in &biome_network.packed_data {
                            blocks_and_biomes_buf.write_i64_be(*packed)?;
                        }
                    }

                    if version == &JavaMinecraftVersion::V_1_21_5 {
                        let block_storage_len = block_network.packed_data.len() as i32;
                        let biome_storage_len = biome_network.packed_data.len() as i32;
                        zero_bytes_count += VarInt(block_storage_len).written_size()
                            + VarInt(biome_storage_len).written_size();
                    }
                }

                if version == &JavaMinecraftVersion::V_1_21_5 && zero_bytes_count > 0 {
                    blocks_and_biomes_buf.resize(blocks_and_biomes_buf.len() + zero_bytes_count, 0);
                }

                write.write_var_int(&blocks_and_biomes_buf.len().try_into().map_err(|_| {
                    WritingError::Message(format!(
                        "{} is not representable as a VarInt!",
                        blocks_and_biomes_buf.len()
                    ))
                })?)?;
                write.write_slice(&blocks_and_biomes_buf)?;
            };

            let block_entities = self
                .0
                .pending_block_entities
                .lock()
                .map_err(|_| WritingError::Message("block_entities lock poisoned".into()))?;
            write.write_var_int(&VarInt(block_entities.len() as i32))?;
            for (pos, nbt) in block_entities.iter() {
                let local_xz =
                    ((get_local_cord(pos.0.x) & 0xF) << 4) | (get_local_cord(pos.0.z) & 0xF);

                write.write_u8(local_xz as u8)?;
                write.write_i16_be(pos.0.y as i16)?;

                let id = nbt.get_string("id").map_or(0, |id_str| {
                    let name = id_str.split(':').next_back().unwrap_or(id_str);
                    pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
                        .iter()
                        .position(|&n| n == name)
                        .unwrap_or(0)
                });
                let remapped_id = pumpkin_data::block_entity_type_id_remap::remap_block_entity_type_id_for_version(id as u32, *version);

                write.write_var_int(&VarInt(remapped_id as i32))?;

                let mut client_nbt = nbt.clone();
                client_nbt.child_tags.remove("id");
                client_nbt.child_tags.remove("x");
                client_nbt.child_tags.remove("y");
                client_nbt.child_tags.remove("z");
                client_nbt.child_tags.remove("LootTable");
                client_nbt.child_tags.remove("LootTableSeed");
                client_nbt.child_tags.remove("PumpkinCustomData");
                client_nbt.child_tags.remove("BukkitValues");
                write_compound_nbt(&mut write, client_nbt, *version)?;
            }

            {
                // Light masks include sections from -1 (below world) to num_sections (above world)
                // This means we need to account for 2 extra sections in the bitset
                let light_engine = self
                    .0
                    .light_engine
                    .lock()
                    .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;
                let num_sections = light_engine.sky_light.len();

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

                // Bit num_sections+1 represents the section above the world (always empty)
                sky_light_empty_mask |= 1 << (num_sections + 1);
                block_light_empty_mask |= 1 << (num_sections + 1);

                if version < &JavaMinecraftVersion::V_1_20_2 {
                    write.write_bool(true)?; // trust edges (removed in 1.20.2)
                }

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
            }
            return Ok(());
        }

        if version >= &JavaMinecraftVersion::V_1_9 {
            // 1.9 to 1.17.1 chunk serialization
            write.write_i32_be(self.0.x)?;
            write.write_i32_be(self.0.z)?;

            if version < &JavaMinecraftVersion::V_1_17 {
                write.write_bool(true)?; // full chunk
            }
            if version == &JavaMinecraftVersion::V_1_16
                || version == &JavaMinecraftVersion::V_1_16_1
            {
                write.write_bool(true)?; // ignore old data
            }

            let block_sections =
                self.0.section.block_sections.read().map_err(|_| {
                    WritingError::Message("block_sections read lock poisoned".into())
                })?;
            let biome_sections =
                self.0.section.biome_sections.read().map_err(|_| {
                    WritingError::Message("biome_sections read lock poisoned".into())
                })?;
            let light_engine = self
                .0
                .light_engine
                .lock()
                .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

            let base_section = (0 - self.0.section.min_y).max(0) as usize / 16;

            let mut chunk_mask = 0u32;
            for i in 0..16 {
                let section_idx = base_section + i;
                if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air()
                {
                    chunk_mask |= 1 << i;
                }
            }

            if version >= &JavaMinecraftVersion::V_1_17 {
                write.write_bitset(&BitSet(Box::new([chunk_mask as i64])))?;
            } else {
                write.write_var_int(&VarInt(chunk_mask as i32))?;
            }

            if version >= &JavaMinecraftVersion::V_1_14 {
                let heightmaps = self
                    .0
                    .heightmap
                    .lock()
                    .map_err(|_| WritingError::Message("heightmap lock poisoned".into()))?;
                let mut comp = pumpkin_nbt::compound::NbtCompound::new();
                if let Some(ref ws) = heightmaps.world_surface {
                    comp.put(
                        "WORLD_SURFACE",
                        pumpkin_nbt::tag::NbtTag::LongArray(ws.to_vec()),
                    );
                }
                if let Some(ref mb) = heightmaps.motion_blocking {
                    comp.put(
                        "MOTION_BLOCKING",
                        pumpkin_nbt::tag::NbtTag::LongArray(mb.to_vec()),
                    );
                }
                if let Some(ref mbnl) = heightmaps.motion_blocking_no_leaves {
                    comp.put(
                        "MOTION_BLOCKING_NO_LEAVES",
                        pumpkin_nbt::tag::NbtTag::LongArray(mbnl.to_vec()),
                    );
                }
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
                                let remapped =
                                    remap_block_state_for_version(raw_state.as_u16(), *version);
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
                        let bits_per_entry =
                            (encompassing_bits(unique_states.len()) as usize).max(4);
                        if bits_per_entry <= 8 {
                            data_buf.write_u8(bits_per_entry as u8)?;
                            data_buf.write_var_int(&VarInt(unique_states.len() as i32))?;
                            for state in &unique_states {
                                data_buf.write_var_int(&VarInt(i32::from(*state)))?;
                            }
                            let indices: Vec<u32> = state_ids
                                .iter()
                                .map(|s| {
                                    unique_states.iter().position(|u| u == s).unwrap_or(0) as u32
                                })
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
                        let direct_indices: Vec<u32> =
                            state_ids.iter().map(|&s| u32::from(s)).collect();
                        let packed = if version >= &JavaMinecraftVersion::V_1_16 {
                            pack_modern_data(&direct_indices, direct_bpe)
                        } else {
                            pack_legacy_data(&direct_indices, direct_bpe)
                        };
                        data_buf.write_list(&packed, |buf, &p| buf.write_i64_be(p))?;
                    }

                    if version < &JavaMinecraftVersion::V_1_14 {
                        let block_light =
                            get_light_bytes(light_engine.block_light.get(section_idx), 0);
                        data_buf.write_slice(&block_light)?;
                        let sky_light =
                            get_light_bytes(light_engine.sky_light.get(section_idx), 15);
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

            let block_entities = self
                .0
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
            return Ok(());
        }

        if version == &JavaMinecraftVersion::V_1_8 {
            // 1.8 chunk serialization
            write.write_i32_be(self.0.x)?;
            write.write_i32_be(self.0.z)?;
            write.write_bool(true)?; // full chunk

            let block_sections =
                self.0.section.block_sections.read().map_err(|_| {
                    WritingError::Message("block_sections read lock poisoned".into())
                })?;
            let biome_sections =
                self.0.section.biome_sections.read().map_err(|_| {
                    WritingError::Message("biome_sections read lock poisoned".into())
                })?;
            let light_engine = self
                .0
                .light_engine
                .lock()
                .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

            let base_section = (0 - self.0.section.min_y).max(0) as usize / 16;

            let mut chunk_mask = 0u16;
            for i in 0..16 {
                let section_idx = base_section + i;
                if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air()
                {
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
                                let remapped =
                                    remap_block_state_for_version(state_id.as_u16(), *version);
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
            return Ok(());
        }

        // 1.7.x chunk serialization
        write.write_i32_be(self.0.x)?;
        write.write_i32_be(self.0.z)?;
        write.write_bool(true)?; // full chunk

        let block_sections = self
            .0
            .section
            .block_sections
            .read()
            .map_err(|_| WritingError::Message("block_sections read lock poisoned".into()))?;
        let biome_sections = self
            .0
            .section
            .biome_sections
            .read()
            .map_err(|_| WritingError::Message("biome_sections read lock poisoned".into()))?;
        let light_engine = self
            .0
            .light_engine
            .lock()
            .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

        let base_section = (0 - self.0.section.min_y).max(0) as usize / 16;

        let mut chunk_mask = 0u16;
        for i in 0..16 {
            let section_idx = base_section + i;
            if section_idx < block_sections.len() && !block_sections[section_idx].has_only_air() {
                chunk_mask |= 1 << i;
            }
        }
        write.write_u16_be(chunk_mask)?;
        write.write_u16_be(0)?; // extended chunk mask

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
                            let remapped =
                                remap_block_state_for_version(state_id.as_u16(), *version);
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

        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&raw_buf)?;
        let compressed = encoder.finish()?;
        write.write_i32_be(compressed.len() as i32)?;
        write.write_slice(&compressed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_world::chunk::ChunkData;

    #[test]
    fn chunk_data_all_versions() {
        let chunk = ChunkData::empty(0, 0);
        let packet = CChunkData(&chunk);

        let versions = [
            JavaMinecraftVersion::V_1_7_2,
            JavaMinecraftVersion::V_1_7_6,
            JavaMinecraftVersion::V_1_8,
            JavaMinecraftVersion::V_1_9,
            JavaMinecraftVersion::V_1_12_2,
            JavaMinecraftVersion::V_1_13_2,
            JavaMinecraftVersion::V_1_14_4,
            JavaMinecraftVersion::V_1_15_2,
            JavaMinecraftVersion::V_1_16_1,
            JavaMinecraftVersion::V_1_16_4,
            JavaMinecraftVersion::V_1_17_1,
            JavaMinecraftVersion::V_1_18_2,
            JavaMinecraftVersion::V_1_19_4,
            JavaMinecraftVersion::V_1_20_2,
            JavaMinecraftVersion::V_1_21_4,
            JavaMinecraftVersion::V_1_21_5,
            JavaMinecraftVersion::V_26_1,
            JavaMinecraftVersion::V_26_2,
        ];

        for version in versions {
            let mut buf = Vec::new();
            let id = CChunkData::to_id(version);
            assert_ne!(id, -1, "Packet ID for version {version:?} must be valid");
            assert!(
                packet.write_packet_data(&mut buf, &version).is_ok(),
                "Failed to serialize chunk data for version {version:?}"
            );
            assert!(
                !buf.is_empty(),
                "Serialized buffer must not be empty for version {version:?}"
            );
        }
    }

    #[test]
    fn populated_chunk_data_all_versions() {
        let chunk = ChunkData::empty(0, 0);
        chunk
            .section
            .set_block_absolute_y(0, 64, 0, pumpkin_data::Block::STONE.default_state.id);
        chunk
            .section
            .set_block_absolute_y(1, 64, 1, pumpkin_data::Block::DIRT.default_state.id);

        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        nbt.put_string("id", "minecraft:chest".to_string());
        chunk.pending_block_entities.lock().unwrap().insert(
            pumpkin_util::math::position::BlockPos(pumpkin_util::math::vector3::Vector3::new(
                0, 64, 0,
            )),
            nbt,
        );

        let packet = CChunkData(&chunk);

        let versions = [
            JavaMinecraftVersion::V_1_7_2,
            JavaMinecraftVersion::V_1_7_6,
            JavaMinecraftVersion::V_1_8,
            JavaMinecraftVersion::V_1_9,
            JavaMinecraftVersion::V_1_12_2,
            JavaMinecraftVersion::V_1_13_2,
            JavaMinecraftVersion::V_1_14_4,
            JavaMinecraftVersion::V_1_15_2,
            JavaMinecraftVersion::V_1_16_1,
            JavaMinecraftVersion::V_1_16_4,
            JavaMinecraftVersion::V_1_17_1,
            JavaMinecraftVersion::V_1_18_2,
            JavaMinecraftVersion::V_1_19_4,
            JavaMinecraftVersion::V_1_20_2,
            JavaMinecraftVersion::V_1_21_4,
            JavaMinecraftVersion::V_1_21_5,
            JavaMinecraftVersion::V_26_1,
            JavaMinecraftVersion::V_26_2,
        ];

        for version in versions {
            let mut buf = Vec::new();
            let id = CChunkData::to_id(version);
            assert_ne!(id, -1, "Packet ID for version {version:?} must be valid");
            assert!(
                packet.write_packet_data(&mut buf, &version).is_ok(),
                "Failed to serialize populated chunk data for version {version:?}"
            );
            assert!(
                !buf.is_empty(),
                "Serialized buffer must not be empty for version {version:?}"
            );
        }
    }
}
