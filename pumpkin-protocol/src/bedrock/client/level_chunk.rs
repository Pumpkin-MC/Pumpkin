use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
const VERSION: u8 = 9;

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
        VarInt(self.chunk.x).write(writer)?;
        VarInt(self.chunk.z).write(writer)?;

        VarInt(self.dimension).write(writer)?;
        let sub_chunk_count = self.chunk.section.sections.len() as u32;
        assert_eq!(sub_chunk_count, 24);
        VarUInt(sub_chunk_count).write(writer)?;
        self.cache_enabled.write(writer)?;

        let mut chunk_data = Vec::new();
        let data_write = &mut chunk_data;

        let min_y = (self.chunk.section.min_y >> 4) as i8;

        // Blocks
        for (i, sub_chunk) in self.chunk.section.sections.iter().enumerate() {
            // Version 9
            // [version:byte][num_storages:byte][sub_chunk_index:byte][block storage1]...[blockStorageN]
            let y = i as i8 + min_y;
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, y as _])?;
            let network_repr = sub_chunk.block_states.convert_be_network();
            (network_repr.bits_per_entry << 1 | 1).write(data_write)?;

            for data in network_repr.packed_data {
                data.write(data_write)?;
            }

            match network_repr.palette {
                NetworkPalette::Single(id) => {
                    VarInt(id as i32).write(data_write)?;
                }
                NetworkPalette::Indirect(palette) => {
                    VarInt(palette.len() as i32).write(data_write)?;
                    for id in palette {
                        VarInt(id as i32).write(data_write)?;
                    }
                }
                NetworkPalette::Direct => (),
            }
        }

        // Biomes
        for i in 0..sub_chunk_count {
            let num_storages = 1;
            let y = i as i8 + min_y;
            data_write.write_all(&[VERSION, num_storages, y as _])?;

            for _ in 0..num_storages {
                1u8.write(data_write)?;
                VarInt(0).write(data_write)?;
            }
        }

        data_write.write_all(&[0])?;

        VarUInt(chunk_data.len() as u32).write(writer)?;
        writer.write_all(&chunk_data)
    }
}
