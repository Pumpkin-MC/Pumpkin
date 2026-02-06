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

impl PacketWrite for CLevelChunk<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.chunk.x).write(writer)?;
        VarInt(self.chunk.z).write(writer)?;

        VarInt(self.dimension).write(writer)?;
        let sub_chunk_count = self.chunk.section.count as u32;
        debug_assert_eq!(sub_chunk_count, 24);
        VarUInt(sub_chunk_count).write(writer)?;
        self.cache_enabled.write(writer)?;

        let mut chunk_data = Vec::new();
        let data_write = &mut chunk_data;

        let block_sections = self.chunk.section.block_sections.read().unwrap();
        let min_y_section = (self.chunk.section.min_y >> 4) as i8;

        for (i, block_palette) in block_sections.iter().enumerate() {
            // Version 9: [version:byte][num_storages:byte][sub_chunk_index:byte]
            let y = (i as i8) + min_y_section;
            let num_storages = 1;
            data_write.write_all(&[VERSION, num_storages, y as u8])?;

            let network_repr = block_palette.convert_be_network();

            (network_repr.bits_per_entry << 1 | 1).write(data_write)?;

            for data in network_repr.packed_data {
                data.write(data_write)?;
            }

            match network_repr.palette {
                NetworkPalette::Single(id) => {
                    VarInt(i32::from(id)).write(data_write)?;
                }
                NetworkPalette::Indirect(palette) => {
                    VarInt(palette.len() as i32).write(data_write)?;
                    for id in palette {
                        VarInt(i32::from(id)).write(data_write)?;
                    }
                }
                NetworkPalette::Direct => (),
            }
        }

        for i in 0..self.chunk.section.count {
            let num_storages = 1;
            let y = (i as i8) + min_y_section;
            data_write.write_all(&[VERSION, num_storages, y as u8])?;

            for _ in 0..num_storages {
                1u8.write(data_write)?;
                // TODO
                VarInt(0).write(data_write)?;
            }
        }

        data_write.write_all(&[0])?;

        VarUInt(chunk_data.len() as u32).write(writer)?;
        writer.write_all(&chunk_data)
    }
}
