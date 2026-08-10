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
        // Optional sub-chunk request limit. Pumpkin sends complete chunks.
        false.write(writer)?;
        self.cache_enabled.write(writer)?;
        // Blob IDs are present in 26.40 even when client caching is disabled.
        VarUInt(0).write(writer)?;

        let mut chunk_data = Vec::new();
        let data_write = &mut chunk_data;

        let block_sections = self
            .chunk
            .section
            .block_sections
            .read()
            .map_err(|_| Error::other("block_sections read lock poisoned"))?;
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

        let biome_sections = self
            .chunk
            .section
            .biome_sections
            .read()
            .map_err(|_| Error::other("biome_sections read lock poisoned"))?;

        for biome_palette in biome_sections.iter() {
            let network_repr = biome_palette.convert_be_network();

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

        data_write.write_all(&[0])?;

        VarUInt(chunk_data.len() as u32).write(writer)?;
        writer.write_all(&chunk_data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64},
    };

    use pumpkin_data::chunk::ChunkStatus;
    use pumpkin_world::{
        chunk::{ChunkData, ChunkHeightmaps, ChunkLight, ChunkSections},
        tick::scheduler::ChunkTickScheduler,
    };

    use super::CLevelChunk;
    use crate::serial::PacketWrite;

    fn read_var_uint(data: &[u8], offset: &mut usize) -> u32 {
        let mut value = 0;
        for shift in (0..35).step_by(7) {
            let byte = data[*offset];
            *offset += 1;
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
        }
        panic!("VarUInt is too long");
    }

    #[test]
    fn biomes_follow_subchunks_without_subchunk_headers() {
        let chunk = ChunkData {
            section: ChunkSections::new(24, -64),
            heightmap: Mutex::new(ChunkHeightmaps::default()),
            x: 0,
            z: 0,
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks: ChunkTickScheduler::default(),
            pending_block_entities: Mutex::default(),
            light_engine: Mutex::new(ChunkLight::default()),
            light_populated: AtomicBool::new(false),
            status: ChunkStatus::Full,
            blending_data: None,
            dirty: AtomicBool::new(false),
            inhabited_time: AtomicU64::new(0),
        };
        let mut encoded = Vec::new();
        CLevelChunk {
            dimension: 0,
            cache_enabled: false,
            chunk: &chunk,
        }
        .write(&mut encoded)
        .unwrap();

        let mut offset = 0;
        for _ in 0..3 {
            read_var_uint(&encoded, &mut offset);
        }
        assert_eq!(read_var_uint(&encoded, &mut offset), 24);
        assert_eq!(encoded[offset], 0); // No sub-chunk request limit.
        assert_eq!(encoded[offset + 1], 0); // Cache disabled.
        offset += 2;
        assert_eq!(read_var_uint(&encoded, &mut offset), 0);
        let raw_len = read_var_uint(&encoded, &mut offset) as usize;
        let raw = &encoded[offset..];
        assert_eq!(raw.len(), raw_len);

        let mut raw_offset = 0;
        for y in -4i8..20 {
            assert_eq!(&raw[raw_offset..raw_offset + 3], &[9, 1, y as u8]);
            raw_offset += 3;
            assert_eq!(raw[raw_offset], 1); // Single-value block palette.
            raw_offset += 1;
            read_var_uint(raw, &mut raw_offset);
        }
        for _ in 0..24 {
            assert_eq!(raw[raw_offset], 1); // No version/storage/Y prefix.
            raw_offset += 1;
            read_var_uint(raw, &mut raw_offset);
        }
        assert_eq!(raw[raw_offset], 0); // Border block count.
        assert_eq!(raw_offset + 1, raw.len());
    }
}
