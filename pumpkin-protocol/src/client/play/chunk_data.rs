use std::io::Write;

use crate::{
    ClientPacket, VarInt,
    codec::bit_set::BitSet,
    ser::{NetworkWriteExt, WritingError},
};

use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_world::chunk::{
    ChunkData,
    palette::{BlockPalette, NetworkPalette},
};

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

        let mut data_buf = Vec::new();
        let mut light_buf = Vec::new();

        for section in self.0.section.sections.iter() {
            // 2 blocks per byte for block lights
            let chunk_light_len = BlockPalette::VOLUME / 2;
            // TODO: Implement, currently default to full bright
            let chunk_light = vec![0xFFu8; chunk_light_len];

            light_buf.write_var_int(&chunk_light_len.into())?;
            light_buf.write_slice(&chunk_light)?;

            // Block count
            let non_empty_block_count = section.block_states.non_air_block_count() as i16;
            data_buf.write_i16_be(non_empty_block_count)?;

            // This is a bit messy, but we dont have access to VarInt in pumpkin-world
            let network_repr = section.block_states.convert_network();
            data_buf.write_u8_be(network_repr.bits_per_entry)?;
            match network_repr.palette {
                NetworkPalette::Single(registry_id) => {
                    data_buf.write_var_int(&registry_id.into())?;
                }
                NetworkPalette::Indirect(palette) => {
                    data_buf.write_var_int(&palette.len().into())?;
                    for registry_id in palette {
                        data_buf.write_var_int(&registry_id.into())?;
                    }
                }
                NetworkPalette::Direct => {}
            }

            // NOTE: Not updated in wiki; i64 array length is now determined by the bits per entry
            //data_buf.write_var_int(&network_repr.packed_data.len().into())?;
            for packed in network_repr.packed_data {
                data_buf.write_i64_be(packed)?;
            }

            let network_repr = section.biomes.convert_network();
            data_buf.write_u8_be(network_repr.bits_per_entry)?;
            match network_repr.palette {
                NetworkPalette::Single(registry_id) => {
                    data_buf.write_var_int(&registry_id.into())?;
                }
                NetworkPalette::Indirect(palette) => {
                    data_buf.write_var_int(&palette.len().into())?;
                    for registry_id in palette {
                        data_buf.write_var_int(&registry_id.into())?;
                    }
                }
                NetworkPalette::Direct => {}
            }

            // NOTE: Not updated in wiki; i64 array length is now determined by the bits per entry
            //data_buf.write_var_int(&network_repr.packed_data.len().into())?;
            for packed in network_repr.packed_data {
                data_buf.write_i64_be(packed)?;
            }
        }

        // Chunk data
        write.write_var_int(&data_buf.len().into())?;
        write.write_slice(&data_buf)?;

        // TODO: block entities
        write.write_var_int(&VarInt(0))?;

        // Sky Light Mask
        // All of the chunks, this is not optimal and uses way more data than needed but will be
        // overhauled with a full lighting system.
        write.write_bitset(&BitSet(Box::new([0b01111111111111111111111110])))?;
        // Block Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;
        // Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;
        // Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;

        // Sky light
        write.write_var_int(&self.0.section.sections.len().into())?;
        write.write_slice(&light_buf)?;

        // Block Lighting
        write.write_var_int(&VarInt(0))
    }
}
