use std::io::Write;

use crate::{
    ClientPacket, VarInt,
    codec::bit_set::BitSet,
    ser::{NetworkWriteExt, WritingError},
};

use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_util::math::ceil_log2;
use pumpkin_world::{
    DIRECT_PALETTE_BITS,
    chunk::{ChunkData, SUBCHUNKS_COUNT},
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

        for subchunk in self.0.sections.array_iter_subchunks() {
            let mut chunk_light = [0u8; 2048];
            for i in 0..subchunk.len() {
                // if !block .is_air() {
                //     continue;
                // }
                let index = i / 2;
                let mask = if i % 2 == 1 { 0xF0 } else { 0x0F };
                chunk_light[index] |= mask;
            }

            light_buf.write_var_int(&VarInt(chunk_light.len() as i32))?;
            light_buf.write_slice(&chunk_light)?;

            let non_empty_block_count = subchunk.len() as i16;
            // Block count
            // TODO: write only non empty blocks, so no air and no fluidstate
            data_buf.write_i16_be(non_empty_block_count)?;

            //// Block states

            let palette = &subchunk;
            // TODO: make dynamic block_size work
            // TODO: make direct block_size work
            enum PaletteType {
                Single,
                Indirect(u32),
                // aka IdListPalette
                Direct,
            }
            let palette_type = {
                let palette_bit_len = ceil_log2(palette.len() as u32);
                if palette_bit_len == 0 {
                    PaletteType::Single
                } else if palette_bit_len <= 4 {
                    PaletteType::Indirect(4)
                } else if palette_bit_len <= 8 {
                    PaletteType::Indirect(palette_bit_len as u32)
                } else {
                    PaletteType::Direct
                }

                // TODO: fix indirect palette to work correctly
                // PaletteType::Direct
            };

            match palette_type {
                PaletteType::Single => {
                    data_buf.write_u8_be(0)?;
                    data_buf.write_var_int(&VarInt(*palette.first().unwrap() as i32))?;
                    data_buf.write_var_int(&VarInt(0))?;
                }
                PaletteType::Indirect(block_size) => {
                    // Bits per entry
                    data_buf.write_u8_be(block_size as u8)?;
                    // Palette length
                    data_buf.write_var_int(&VarInt(palette.len() as i32 - 1))?;

                    for id in palette.iter() {
                        // Palette
                        data_buf.write_var_int(&VarInt(*id as i32))?;
                    }

                    // Data array length
                    let data_array_len = subchunk.len().div_ceil(64 / block_size as usize);

                    data_buf.reserve(data_array_len * 8);
                    for block_clump in subchunk.chunks(64 / block_size as usize) {
                        let mut out_long: i64 = 0;
                        for block in block_clump.iter().rev() {
                            let index = palette
                                .iter()
                                .position(|b| b == block)
                                .expect("Its just got added, ofc it should be there");
                            out_long = (out_long << block_size) | (index as i64);
                        }
                        data_buf.write_i64_be(out_long)?;
                    }
                }
                PaletteType::Direct => {
                    // Bits per entry
                    data_buf.write_u8_be(DIRECT_PALETTE_BITS as u8)?;
                    // Data array length
                    let data_array_len = subchunk.len().div_ceil(64 / DIRECT_PALETTE_BITS as usize);
                    data_buf.reserve(data_array_len * 8);
                    for block_clump in subchunk.chunks(64 / DIRECT_PALETTE_BITS as usize) {
                        let mut out_long: i64 = 0;
                        for (i, &block) in block_clump.iter().enumerate() {
                            out_long |= (block as i64) << (i as u32 * DIRECT_PALETTE_BITS);
                        }
                        data_buf.write_i64_be(out_long)?;
                    }
                }
            }

            //// Biomes
            // TODO: make biomes work
            // bits
            data_buf.write_u8_be(0)?;
            data_buf.write_var_int(&VarInt(0))?;
        }

        // Size
        write.write_var_int(&VarInt(data_buf.len() as i32))?;
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
        write.write_var_int(&VarInt(SUBCHUNKS_COUNT as i32))?;
        write.write_slice(&light_buf)?;

        // Block Lighting
        write.write_var_int(&VarInt(0))
    }
}
