use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_nbt::END_ID;
use pumpkin_util::math::position::get_local_cord;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};
use std::io::Write;

#[packet(PLAY_LEVEL_CHUNK_WITH_LIGHT)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl ClientPacket for CChunkData<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_i32_be(self.0.position.x)?;
        // Chunk Z
        write.write_i32_be(self.0.position.y)?;

        let heightmaps = &self.0.heightmap;
        // the heighmap is a map, we put 3 values in so the size is 3
        write.write_var_int(&VarInt(3))?;

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
        // heighmap index
        write.write_var_int(&VarInt(5))?;
        // write long array
        write.write_var_int(&VarInt(heightmaps.motion_blocking_no_leaves.len() as i32))?;
        for mb in &heightmaps.motion_blocking {
            write.write_i64_be(*mb)?;
        }

        {
            let mut blocks_and_biomes_buf = Vec::new();
            for section in &self.0.section.sections {
                // Block count
                let non_empty_block_count = section.block_states.non_air_block_count() as i16;
                blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;

                // This is a bit messy, but we dont have access to VarInt in pumpkin-world
                let network_repr = section.block_states.convert_network();
                blocks_and_biomes_buf.write_u8(network_repr.bits_per_entry)?;
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

                let network_repr = section.biomes.convert_network();
                blocks_and_biomes_buf.write_u8(network_repr.bits_per_entry)?;
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
            write.write_var_int(&blocks_and_biomes_buf.len().try_into().map_err(|_| {
                WritingError::Message(format!(
                    "{} is not representable as a VarInt!",
                    blocks_and_biomes_buf.len()
                ))
            })?)?;
            write.write_slice(&blocks_and_biomes_buf)?;
        }

        // TODO: block entities
        write.write_var_int(&VarInt(self.0.block_entities.len() as i32))?;
        for block_entity in self.0.block_entities.values() {
            let block_entity = &block_entity;
            let chunk_data_nbt = block_entity.chunk_data_nbt();
            let pos = block_entity.get_position();
            let block_entity_id = block_entity.get_id();
            let local_xz = (get_local_cord(pos.0.x) << 4) | get_local_cord(pos.0.z);
            write.write_u8(local_xz as u8)?;
            write.write_i16_be(pos.0.y as i16)?;
            write.write_var_int(&VarInt(block_entity_id as i32))?;
            if let Some(chunk_data_nbt) = chunk_data_nbt {
                write.write_nbt(&chunk_data_nbt.into())?;
            } else {
                write.write_u8(END_ID)?;
            }
        }

        {
            // todo: these masks are 64 bits long, we should use a bitset instead of a u64
            //  in higher maps
            let mut sky_light_empty_mask = 0;
            let mut block_light_empty_mask = 0;
            let mut sky_light_mask = 0;
            let mut block_light_mask = 0;
            for light_index in 0..self.0.light_engine.sky_light.len() {
                if let LightContainer::Full(_) = &self.0.light_engine.sky_light[light_index] {
                    sky_light_mask |= 1 << light_index;
                } else {
                    sky_light_empty_mask |= 1 << light_index;
                }

                if let LightContainer::Full(_) = &self.0.light_engine.block_light[light_index] {
                    block_light_mask |= 1 << light_index;
                } else {
                    block_light_empty_mask |= 1 << light_index;
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

            let light_data_size: VarInt = LightContainer::ARRAY_SIZE.try_into().unwrap();
            // Sky light
            write.write_var_int(&VarInt(sky_light_mask.count_ones() as i32))?;
            for light_index in 0..self.0.light_engine.sky_light.len() {
                if let LightContainer::Full(data) = &self.0.light_engine.sky_light[light_index] {
                    write.write_var_int(&light_data_size)?;
                    write.write_slice(data)?;
                }
            }

            // Block Light
            write.write_var_int(&VarInt(block_light_mask.count_ones() as i32))?;
            for light_index in 0..self.0.light_engine.block_light.len() {
                if let LightContainer::Full(data) = &self.0.light_engine.block_light[light_index] {
                    write.write_var_int(&light_data_size)?;
                    write.write_slice(data)?;
                }
            }
        }
        Ok(())
    }
}
