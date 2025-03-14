use crate::{
    ClientPacket, VarInt,
    codec::bit_set::BitSet,
    ser::{NetworkWrite, WritingError},
};

use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::packet;
use pumpkin_world::{
    DIRECT_PALETTE_BITS,
    chunk::{ChunkData, SUBCHUNKS_COUNT},
};

#[packet(PLAY_LEVEL_CHUNK_WITH_LIGHT)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl ClientPacket for CChunkData<'_> {
    fn write(&self, write: impl NetworkWrite) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_i32_be(self.0.position.x)?;
        // Chunk Z
        write.write_i32_be(self.0.position.z)?;

        let mut heightmap_nbt = Vec::new();
        pumpkin_nbt::serializer::to_bytes_unnamed(&self.0.heightmap, &mut heightmap_nbt).unwrap();
        // Heightmaps
        write.write_slice(&heightmap_nbt);

        let mut data_buf = Vec::new();
        for subchunk in self.0.subchunks.array_iter() {
            let block_count = subchunk.len() as i16;
            // Block count
            data_buf.write_i16_be(block_count)?;
            //// Block states

            let palette = &subchunk;
            // TODO: make dynamic block_size work
            // TODO: make direct block_size work
            enum PaletteType {
                Indirect(u32),
                Direct,
            }
            let palette_type = {
                let palette_bit_len = 64 - (palette.len() as i64 - 1).leading_zeros();
                if palette_bit_len > 8 {
                    PaletteType::Direct
                } else if palette_bit_len > 3 {
                    PaletteType::Indirect(palette_bit_len)
                } else {
                    PaletteType::Indirect(4)
                }
                // TODO: fix indirect palette to work correctly
                // PaletteType::Direct
            };

            match palette_type {
                PaletteType::Indirect(block_size) => {
                    // Bits per entry
                    data_buf.write_u8_be(block_size as u8)?;
                    // Palette length
                    data_buf.write_var_int(&VarInt(palette.len() as i32))?;

                    for id in palette.iter() {
                        // Palette
                        data_buf.write_var_int(&VarInt(*id as i32))?;
                    }

                    // Data array length
                    let data_array_len = subchunk.len().div_ceil(64 / block_size as usize);
                    data_buf.write_var_int(&VarInt(data_array_len as i32))?;

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
                    data_buf.write_var_int(&VarInt(data_array_len as i32))?;

                    data_buf.reserve(data_array_len * 8);
                    for block_clump in subchunk.chunks(64 / DIRECT_PALETTE_BITS as usize) {
                        let mut out_long: i64 = 0;
                        let mut shift = 0;
                        for block in block_clump {
                            out_long |= (*block as i64) << shift;
                            shift += DIRECT_PALETTE_BITS;
                        }
                        data_buf.write_i64_be(out_long)?;
                    }
                }
            }

            //// Biomes
            // TODO: make biomes work
            data_buf.write_u8_be(0)?;
            // This seems to be the biome
            data_buf.write_var_int(&VarInt(10))?;
            data_buf.write_var_int(&VarInt(0))?;
        }

        // Size
        write.write_var_int(&VarInt(data_buf.len() as i32))?;
        // Data
        write.write_slice(&data_buf)?;

        // TODO: block entities
        write.write_var_int(&VarInt(0))?;

        // Sky Light Mask
        // All of the chunks, this is not optimal and uses way more data than needed but will be
        // overhauled with full lighting system.
        write.write_bitset(&BitSet(Box::new([0b01111111111111111111111110])))?;
        // Block Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;
        // Empty Sky Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;
        // Empty Block Light Mask
        write.write_bitset(&BitSet(Box::new([0])))?;

        write.write_var_int(&VarInt(SUBCHUNKS_COUNT as i32))?;
        for chunk in self.0.subchunks.array_iter() {
            let mut chunk_light = [0u8; 2048];
            for (i, _) in chunk.iter().enumerate() {
                // if !block .is_air() {
                //     continue;
                // }
                let index = i / 2;
                let mask = if i % 2 == 1 { 0xF0 } else { 0x0F };
                chunk_light[index] |= mask;
            }

            write.write_var_int(&VarInt(chunk_light.len() as i32))?;
            write.write_slice(&chunk_light)?;
        }

        // Block Lighting
        write.write_var_int(&VarInt(0))
    }
}
