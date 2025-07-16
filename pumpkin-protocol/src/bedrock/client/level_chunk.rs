use std::io::{Error, Write};

use pumpkin_macros::packet;
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
    pub chunk: &'a ChunkData,
}

impl<'a> PacketWrite for CLevelChunk<'a> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.chunk.position.x).write(writer)?;
        VarInt(self.chunk.position.y).write(writer)?;

        let sub_chunk_count = self.chunk.section.sections.len() as u32;
        VarUInt(sub_chunk_count).write(writer)?;
        VarInt(self.dimension).write(writer)?;
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
                let format = 1 << 1 | 1;
                data_write.write_all(&[format])?;

                let blocks_per_word = 32 / 1;

                for _ in 0..(CHUNK_SIZE / blocks_per_word) {
                    data_write.write_all(&u32::MAX.to_le_bytes())?;
                }

                VarInt(2).write(data_write)?;

                VarInt(0).write(data_write)?;
                VarInt(1).write(data_write)?;
            }
        }

        // Biomes
        for i in 0..sub_chunk_count {
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, i as u8])?;

            for _ in 0..num_storages {
                let format = 1 << 1 | 1;
                data_write.write_all(&[format])?;

                let blocks_per_word = 32 / 1;

                for _ in 0..(BIOME_SIZE / blocks_per_word) {
                    data_write.write_all(&0u32.to_le_bytes())?;
                }

                VarInt(1).write(data_write)?;
                VarInt(1).write(data_write)?;
            }
        }

        data_write.write(&[0])?;

        VarUInt(chunk_data.len() as u32).write(writer)?;
        writer.write_all(&chunk_data)
    }
}
