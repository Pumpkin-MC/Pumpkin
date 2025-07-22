use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::encompassing_bits;
use pumpkin_world::chunk::{palette::NetworkPalette, ChunkData};

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

const VERSION: u8 = 9;
const _CHUNK_SIZE: usize = 4096;
const BIOME_SIZE: usize = 64;

#[packet(58)]
pub struct CLevelChunk<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/LevelChunkPacket.html
    pub dimension: i32,
    pub cache_enabled: bool,

    // https://gist.github.com/Tomcc/a96af509e275b1af483b25c543cfbf37
    // https://github.com/Mojang/bedrock-protocol-docs/blob/main/additional_docs/SubChunk%20Request%20System%20v1.18.10.md
    pub chunk: &'a ChunkData,
}

impl<'a> PacketWrite for CLevelChunk<'a> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.chunk.position.x).write(writer)?;
        VarInt(self.chunk.position.y).write(writer)?;

        VarInt(self.dimension).write(writer)?;
        let sub_chunk_count = self.chunk.section.sections.len() as u32;
        VarUInt(sub_chunk_count).write(writer)?;
        self.cache_enabled.write(writer)?;

        let mut chunk_data = Vec::new();
        let data_write = &mut chunk_data;

        // Blocks
        for (i, sub_chunk) in self.chunk.section.sections.iter().enumerate() {
            // Version 9
            // [version:byte][num_storages:byte][sub_chunk_index:byte][block storage1]...[blockStorageN]
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, ((i as i8) - 4) as u8])?;
            let network_repr = sub_chunk.block_states.convert_be_network();
            (network_repr.bits_per_entry << 1 | 1).write(data_write)?;

            for data in network_repr.packed_data.iter() {
                data.write(data_write)?;
            }

            match network_repr.palette {
                NetworkPalette::Single(registry_id) => {
                    //println!("bits: {}, id: {}", network_repr.bits_per_entry, registry_id);
                    VarInt(!registry_id as i32).write(data_write)?;
                }
                NetworkPalette::Indirect(palette) => {
                    VarInt(palette.len() as i32).write(data_write)?;
                    for mut id in palette {
                        if id == 0 { id = u16::MAX;} else {
                            id = id;
                        }
                        VarInt(id as i32).write(data_write)?;
                    }
                }
                NetworkPalette::Direct => ()
            }
        }

        // Biomes
        for i in 0..sub_chunk_count {
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, ((i as i8) - 4) as u8])?;

            for _ in 0..num_storages {
                encode_storage(data_write, 1, BIOME_SIZE)?;
            }
        }

        data_write.write_all(&[0])?;

        VarUInt(chunk_data.len() as u32).write(writer)?;
        writer.write_all(&chunk_data)
    }
}

fn encode_storage<W: Write>(
    writer: &mut W,
    palette_size: i32,
    indices_len: usize,
) -> Result<(), Error> {
    if palette_size < 2 {
        1u8.write(writer)?;
        VarInt(0).write(writer)?;
        return Ok(());
    }
    
    let bits_per_index: u8 = encompassing_bits(palette_size as _);

    let format = bits_per_index << 1 | 1;
    format.write(writer)?;

    let blocks_per_word = 32 / bits_per_index as usize;

    for _ in 0..(indices_len / blocks_per_word) {
        writer.write_all(&u32::MAX.to_le_bytes())?;
    }

    VarInt(palette_size).write(writer)?;

    for i in 0..palette_size {
        VarInt(i).write(writer)?;
    }

    Ok(())
}
