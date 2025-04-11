use std::io::Write;

use crate::{
    ClientPacket, VarInt,
    codec::bit_set::BitSet,
    ser::{NetworkWriteExt, WritingError},
};

use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};

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

        for (i, section) in self.0.section.sections.iter().enumerate() {
            // Write sky light
            if let Some(sky_light) = &section.sky_light {
                let mut buf = Vec::new();
                buf.write_var_int(&sky_light.len().try_into().map_err(|_| {
                    WritingError::Message("sky_light not representable as a VarInt!".to_string())
                })?)?;
                buf.write_slice(sky_light)?;
                sky_light_buf.push(buf);
                sky_light_mask |= 1 << i;
            } else {
                sky_light_empty_mask |= 1 << i;
            }

            // Write block light
            if let Some(block_light) = &section.block_light {
                let mut buf = Vec::new();
                buf.write_var_int(&block_light.len().try_into().map_err(|_| {
                    WritingError::Message("block_light not representable as a VarInt!".to_string())
                })?)?;
                buf.write_slice(block_light)?;
                block_light_buf.push(buf);
                block_light_mask |= 1 << i;
            } else {
                block_light_empty_mask |= 1 << i;
            }

            // Block count
            let non_empty_block_count = section.block_states.non_air_block_count() as i16;
            blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;

            // This is a bit messy, but we dont have access to VarInt in pumpkin-world
            let network_repr = section.block_states.convert_network();
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

            let network_repr = section.biomes.convert_network();
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

        // Chunk data
        write.write_var_int(&blocks_and_biomes_buf.len().try_into().map_err(|_| {
            WritingError::Message(format!(
                "{} is not representable as a VarInt!",
                blocks_and_biomes_buf.len()
            ))
        })?)?;
        write.write_slice(&blocks_and_biomes_buf)?;

        // TODO: block entities
        write.write_var_int(&VarInt(0))?;

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
