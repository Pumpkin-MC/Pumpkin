use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::encompassing_bits;
use pumpkin_world::chunk::ChunkData;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

const VERSION: u8 = 9;
const CHUNK_SIZE: usize = 4096;
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
        for i in 0..sub_chunk_count {
            // Version 9
            // [version:byte][num_storages:byte][sub_chunk_index:byte][block storage1]...[blockStorageN]
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, i as u8])?;

            for _ in 0..num_storages {
                encode_storage(data_write, 2, CHUNK_SIZE)?;
            }
        }

        // Biomes
        for i in 0..sub_chunk_count {
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, i as u8])?;

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
