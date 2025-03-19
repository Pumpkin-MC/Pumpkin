use crate::{ClientPacket, VarInt, bytebuf::ByteBufMut, codec::bit_set::BitSet};

use bytes::{BufMut, BytesMut};
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
    fn write(&self, buf: &mut impl BufMut) {
        // Chunk X
        buf.put_i32(self.0.position.x);
        // Chunk Z
        buf.put_i32(self.0.position.z);

        pumpkin_nbt::serializer::to_bytes_unnamed(&self.0.heightmap, &mut buf.writer()).unwrap();

        let mut data_buf = BytesMut::new();
        let mut light_buf = BytesMut::new();
        self.0.blocks.array_iter_subchunks().for_each(|subchunk| {
            let mut chunk_light = [0u8; 2048];
            for i in 0..subchunk.len() {
                // if !block .is_air() {
                //     continue;
                // }
                let index = i / 2;
                let mask = if i % 2 == 1 { 0xF0 } else { 0x0F };
                chunk_light[index] |= mask;
            }

            light_buf.put_var_int(&VarInt(chunk_light.len() as i32));
            light_buf.put_slice(&chunk_light);

            let block_count = subchunk.len() as i16;
            // Block count
            data_buf.put_i16(block_count);
            //// Block states

            let palette = &subchunk;
            // TODO: make dynamic block_size work
            // TODO: make direct block_size work
            enum PaletteType {
                Single,
                Indirect(u32),
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
                    data_buf.put_u8(0);
                    data_buf.put_var_int(&VarInt(*palette.first().unwrap() as i32));
                    data_buf.put_var_int(&VarInt(0));
                }
                PaletteType::Indirect(block_size) => {
                    // Bits per entry
                    data_buf.put_u8(block_size as u8);
                    // Palette length
                    data_buf.put_var_int(&VarInt(palette.len() as i32 - 1));

                    palette.iter().for_each(|id| {
                        // Palette
                        data_buf.put_var_int(&VarInt(*id as i32));
                    });
                    // Data array length
                    let data_array_len = subchunk.len().div_ceil(64 / block_size as usize);
                    data_buf.put_var_int(&VarInt(data_array_len as i32));

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
                        data_buf.put_i64(out_long);
                    }
                }
                PaletteType::Direct => {
                    // Bits per entry
                    data_buf.put_u8(DIRECT_PALETTE_BITS as u8);
                    // Data array length
                    let data_array_len = subchunk.len().div_ceil(64 / DIRECT_PALETTE_BITS as usize);
                    data_buf.put_var_int(&VarInt(data_array_len as i32));

                    data_buf.reserve(data_array_len * 8);
                    for block_clump in subchunk.chunks(64 / DIRECT_PALETTE_BITS as usize) {
                        let mut out_long: i64 = 0;
                        for (i, &block) in block_clump.iter().enumerate() {
                            out_long |= (block as i64) << (i as u32 * DIRECT_PALETTE_BITS);
                        }
                        data_buf.put_i64(out_long);
                    }
                }
            }

            //// Biomes
            // TODO: make biomes work
            data_buf.put_u8(0);
            // This seems to be the biome
            data_buf.put_var_int(&VarInt(10));
            data_buf.put_var_int(&VarInt(0));
        });

        // Size
        buf.put_var_int(&VarInt(data_buf.len() as i32));
        // Data
        buf.put_slice(&data_buf);

        // TODO: block entities
        buf.put_var_int(&VarInt(0));

        // Sky Light Mask
        // All of the chunks, this is not optimal and uses way more data than needed but will be
        // overhauled with a full lighting system.
        buf.put_bit_set(&BitSet(VarInt(1), vec![0b01111111111111111111111110]));
        // Block Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));
        // Empty Sky Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), vec![0b0]));
        // Empty Block Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));

        // Sky light
        buf.put_var_int(&VarInt(SUBCHUNKS_COUNT as i32));
        buf.put_slice(&light_buf);

        // Block Lighting
        buf.put_var_int(&VarInt(0));
    }
}
