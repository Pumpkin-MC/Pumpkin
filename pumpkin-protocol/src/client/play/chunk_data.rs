use std::io::Write;

use crate::{
    ClientPacket, VarInt,
    codec::bit_set::BitSet,
    ser::{NetworkWriteExt, WritingError},
};

use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_nbt::END_ID;
use pumpkin_util::math::position::get_local_cord;
use pumpkin_world::chunk::{ChunkData, SubChunk, palette::NetworkPalette};

// TODO: Take some of this data from GENERATION_SETTINGS ?
pub const WORLD_LOWEST_Y: i8 = -64;
const WORLD_HEIGHT: u16 = 384;
const SECTION_SIZE: i8 = 16;
pub const WORLD_HIGHEST_Y: i32 = (WORLD_LOWEST_Y as i32) + (WORLD_HEIGHT as i32);
const SECTION_LOWEST_POPULATED_Y: i8 = WORLD_LOWEST_Y / SECTION_SIZE;
const SECTION_LOWEST_Y: i8 = SECTION_LOWEST_POPULATED_Y - 1;
const SECTION_HIGHEST_Y: i32 = WORLD_HIGHEST_Y / (SECTION_SIZE as i32);
const SECTION_HIGHEST_POPULATED_Y: i32 = SECTION_HIGHEST_Y - 1;
const TOP_SECTION_LIGHT_INDEX: i32 = SECTION_HIGHEST_Y - (SECTION_LOWEST_Y as i32);

#[packet(PLAY_LEVEL_CHUNK_WITH_LIGHT)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl ClientPacket for CChunkData<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_i32_be(self.0.position.x)?;
        // Chunk Z
        write.write_i32_be(self.0.position.z)?;

        let heightmaps = &self.0.heightmap;
        // the heighmap is a map, we put 2 values in so the size is 2
        write.write_var_int(&VarInt(2))?;

        // heighmap index
        write.write_var_int(&VarInt(1))?;
        // write long array
        write.write_var_int(&VarInt(heightmaps.world_surface.len() as i32))?;
        for mb in &heightmaps.world_surface {
            write.write_i64_be(*mb)?;
        }
        // heighmap index
        write.write_var_int(&VarInt(4))?;
        // write long array
        write.write_var_int(&VarInt(heightmaps.motion_blocking.len() as i32))?;
        for mb in &heightmaps.motion_blocking {
            write.write_i64_be(*mb)?;
        }

        let mut blocks_and_biomes_buf = Vec::new();

        let mut sky_light_buf = Vec::new();
        let mut sky_light_empty_mask = 0;
        let mut sky_light_mask = 0;
        let mut block_light_buf = Vec::new();
        let mut block_light_empty_mask = 0;
        let mut block_light_mask = 0;

        for section in self.0.section.sections.iter() {
            let light_index = (section.y as i32) - (SECTION_LOWEST_Y as i32);

            // Write sky light
            if let Some(sky_light) = &section.sky_light {
                let mut buf = Vec::new();
                buf.write_var_int(&sky_light.len().try_into().map_err(|_| {
                    WritingError::Message("sky_light not representable as a VarInt!".to_string())
                })?)?;
                buf.write_slice(sky_light)?;
                sky_light_buf.push(buf);
                sky_light_mask |= 1 << light_index;
            } else {
                sky_light_empty_mask |= 1 << light_index;
            }

            // Write block light
            if let Some(block_light) = &section.block_light {
                let mut buf = Vec::new();
                buf.write_var_int(&block_light.len().try_into().map_err(|_| {
                    WritingError::Message("block_light not representable as a VarInt!".to_string())
                })?)?;
                buf.write_slice(block_light)?;
                block_light_buf.push(buf);
                block_light_mask |= 1 << light_index;
            } else {
                block_light_empty_mask |= 1 << light_index;
            }

            if section.y < SECTION_LOWEST_POPULATED_Y
                || (section.y as i32) > SECTION_HIGHEST_POPULATED_Y
            {
                // Chunks from vanilla region files *might* include 1 section below and 1 section above the world height that stores block light data.
                continue;
            }

            let Some(block_states) = &section.block_states else {
                return Err(WritingError::Message(format!(
                    "Section at y {} of chunk ({}, {}) does not contain any block data! This should never be possible.",
                    self.0.position.z, section.y, self.0.position.x
                )));
            };

            // Block count
            let non_empty_block_count = block_states.non_air_block_count() as i16;
            blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;

            // This is a bit messy, but we dont have access to VarInt in pumpkin-world
            let network_repr = block_states.convert_network();
            blocks_and_biomes_buf.write_u8_be(network_repr.bits_per_entry)?;
            match network_repr.palette {
                NetworkPalette::Single(registry_id) => {
                    blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                }
                NetworkPalette::Indirect(palette) => {
                    blocks_and_biomes_buf.write_var_int(&palette.len().try_into().map_err(
                        |_| {
                            WritingError::Message(format!(
                                "{} is not representable as a VarInt!",
                                palette.len()
                            ))
                        },
                    )?)?;
                    for registry_id in palette {
                        blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                    }
                }
                NetworkPalette::Direct => {}
            }

            for packed in network_repr.packed_data {
                blocks_and_biomes_buf.write_i64_be(packed)?;
            }

            let Some(biomes) = &section.biomes else {
                return Err(WritingError::Message(format!(
                    "Section at y {} of chunk ({}, {}) does not contain any biome data! This should never be possible.",
                    self.0.position.z, section.y, self.0.position.x
                )));
            };

            let network_repr = biomes.convert_network();
            blocks_and_biomes_buf.write_u8_be(network_repr.bits_per_entry)?;
            match network_repr.palette {
                NetworkPalette::Single(registry_id) => {
                    blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                }
                NetworkPalette::Indirect(palette) => {
                    blocks_and_biomes_buf.write_var_int(&palette.len().try_into().map_err(
                        |_| {
                            WritingError::Message(format!(
                                "{} is not representable as a VarInt!",
                                palette.len()
                            ))
                        },
                    )?)?;
                    for registry_id in palette {
                        blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                    }
                }
                NetworkPalette::Direct => {}
            }

            // NOTE: Not updated in wiki; i64 array length is now determined by the bits per entry
            //data_buf.write_var_int(&network_repr.packed_data.len().into())?;
            for packed in network_repr.packed_data {
                blocks_and_biomes_buf.write_i64_be(packed)?;
            }
        }

        if let Some(first_section) = self.0.section.sections.last() {
            if (first_section.y as i32) != SECTION_HIGHEST_Y {
                // The client expects light data for all sections where blocks can be placed plus one above and below that so here we set the light data for the top section if it is not present in the section data

                let max_light = SubChunk::max_sky_light_data();

                let mut buf = Vec::new();
                buf.write_var_int(&max_light.len().try_into().map_err(|_| {
                    WritingError::Message("sky_light not representable as a VarInt!".to_string())
                })?)?;
                buf.write_slice(&max_light)?;
                sky_light_buf.push(buf);

                sky_light_mask |= 1 << TOP_SECTION_LIGHT_INDEX;
                block_light_empty_mask |= 1 << TOP_SECTION_LIGHT_INDEX;
            }
        }

        // Chunk data
        write.write_var_int(&blocks_and_biomes_buf.len().try_into().map_err(|_| {
            WritingError::Message(format!(
                "{} is not representable as a VarInt!",
                blocks_and_biomes_buf.len()
            ))
        })?)?;
        write.write_slice(&blocks_and_biomes_buf)?;

        // TODO: block entities
        write.write_var_int(&VarInt(self.0.block_entities.len() as i32))?;
        for block_entity in self.0.block_entities.values() {
            let chunk_data_nbt = block_entity.chunk_data_nbt();
            let pos = block_entity.get_position();
            let block_entity_id = block_entity.get_id();
            let local_xz = (get_local_cord(pos.0.x) << 4) | get_local_cord(pos.0.z);
            write.write_u8_be(local_xz as u8)?;
            write.write_i16_be(pos.0.y as i16)?;
            write.write_var_int(&VarInt(block_entity_id as i32))?;
            if let Some(chunk_data_nbt) = chunk_data_nbt {
                write.write_nbt(&chunk_data_nbt.into())?;
            } else {
                write.write_u8_be(END_ID)?;
            }
        }

        // Sky Light Mask
        // All of the chunks, this is not optimal and uses way more data than needed but will be
        // overhauled with a full lighting system.

        // Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_mask])))?;
        // Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_mask])))?;
        // Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([sky_light_empty_mask])))?;
        // Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([block_light_empty_mask])))?;

        // Sky light
        write.write_var_int(&VarInt(sky_light_buf.len() as i32))?;
        for sky_buf in sky_light_buf {
            write.write_slice(&sky_buf)?;
        }

        // Block Light
        write.write_var_int(&VarInt(block_light_buf.len() as i32))?;
        for block_buf in block_light_buf {
            write.write_slice(&block_buf)?;
        }
        Ok(())
    }
}
