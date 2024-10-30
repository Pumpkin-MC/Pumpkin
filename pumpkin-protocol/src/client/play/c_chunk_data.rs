use std::collections::HashMap;

use crate::{bytebuf::ByteBuffer, BitSet, ClientPacket, VarInt};
use itertools::Itertools;
use pumpkin_macros::client_packet;
use pumpkin_world::{chunk::ChunkData, DIRECT_PALETTE_BITS};

#[client_packet("play:level_chunk_with_light")]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl<'a> ClientPacket for CChunkData<'a> {
    fn write(&self, buf: &mut crate::bytebuf::ByteBuffer) {
        // Chunk X
        buf.put_i32(self.0.position.x);
        // Chunk Z
        buf.put_i32(self.0.position.z);

        let heightmap_nbt =
            fastnbt::to_bytes_with_opts(&self.0.blocks.heightmap, fastnbt::SerOpts::network_nbt())
                .unwrap();
        // Heightmaps
        buf.put_slice(&heightmap_nbt);

        let mut data_buf = ByteBuffer::empty();
        self.0.blocks.iter_subchunks().for_each(|chunk| {
            let block_count = chunk.iter().filter(|block| !block.is_air()).count() as i16;
            // Block count
            data_buf.put_i16(block_count);
            //// Block states

            let palette = chunk.iter().dedup().collect_vec();
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
                    data_buf.put_u8(block_size as u8);
                    // Palette length
                    data_buf.put_var_int(&VarInt(palette.len() as i32));
                    let mut palette_map = HashMap::new();
                    palette.iter().enumerate().for_each(|(i, id)| {
                        palette_map.insert(*id, i);
                        // Palette
                        data_buf.put_var_int(&VarInt(id.get_id_mojang_repr()));
                    });
                    // Data array length
                    data_buf.put_var_int(&VarInt(
                        chunk.len().div_ceil(64 / block_size as usize) as i32
                    ));
                    for block_clump in chunk.chunks(64 / block_size as usize) {
                        let mut out_long: i64 = 0;
                        for block in block_clump.iter().rev() {
                            let index = palette_map
                                .get(block)
                                .expect("Its just got added, ofc it should be there");
                            out_long = out_long << block_size | (*index as i64);
                        }
                        data_buf.put_i64(out_long);
                    }
                }
                PaletteType::Direct => {
                    // Bits per entry
                    data_buf.put_u8(DIRECT_PALETTE_BITS as u8);
                    // Data array length
                    data_buf.put_var_int(&VarInt(
                        chunk.len().div_ceil(64 / DIRECT_PALETTE_BITS as usize) as i32,
                    ));
                    for block_clump in chunk.chunks(64 / DIRECT_PALETTE_BITS as usize) {
                        let mut out_long: i64 = 0;
                        let mut shift = 0;
                        for block in block_clump {
                            out_long |= (block.get_id() as i64) << shift;
                            shift += DIRECT_PALETTE_BITS;
                        }
                        data_buf.put_i64(out_long);
                    }
                }
            }

            //// Biomes
            // TODO: make biomes work
            data_buf.put_u8(0);
            data_buf.put_var_int(&VarInt(0));
            data_buf.put_var_int(&VarInt(0));
        });

        // Size
        buf.put_var_int(&VarInt(data_buf.buf().len() as i32));
        // Data
        buf.put_slice(data_buf.buf());

        // TODO: block entities
        buf.put_var_int(&VarInt(0));

        if let Some(light) = &self.0.light {
            let (sky_light_mask, empty_sky_light_mask, subchunks) = light.packet_data();

            // Sky Light Mask
            // All of the chunks, this is not optimal and uses way more data than needed but will be
            // overhauled with full lighting system.
            buf.put_bit_set(&BitSet(VarInt(1), &[sky_light_mask]));
            // Block Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));
            // Empty Sky Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[empty_sky_light_mask]));
            // Empty Block Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));

            buf.put_var_int(&subchunks.len().into());
            for subchunk in subchunks.into_iter() {
                buf.put_var_int(&VarInt(subchunk.len() as i32));
                buf.put_slice(subchunk);
            }

            // Block Lighting
            buf.put_var_int(&VarInt(0));
        } else {
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));
            // Block Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));
            // Empty Sky Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));
            // Empty Block Light Mask
            buf.put_bit_set(&BitSet(VarInt(1), &[0]));

            buf.put_var_int(&VarInt(0));
            buf.put_var_int(&VarInt(0));
        }
    }
}
