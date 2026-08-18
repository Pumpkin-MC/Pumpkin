use std::io::{Error, Write};
use xxhash_rust::xxh64::xxh64;

use pumpkin_macros::packet;
use pumpkin_nbt::{Nbt, compound::NbtCompound};
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
    pub block_actors: &'a [NbtCompound],
}

pub type ChunkBlob = (u64, Vec<u8>);
pub type EncodedChunk = (Vec<u8>, Vec<ChunkBlob>);

fn encode_block_actors(block_actors: &[NbtCompound]) -> Result<Vec<u8>, Error> {
    let mut encoded = Vec::new();
    for block_actor in block_actors {
        encoded.write_all(&Nbt::from(block_actor.clone()).write_bedrock())?;
    }
    Ok(encoded)
}

impl CLevelChunk<'_> {
    pub fn encode_chunk(
        chunk: &ChunkData,
        dimension: i32,
        cache_enabled: bool,
        block_actors: &[NbtCompound],
    ) -> Result<EncodedChunk, Error> {
        let mut writer = Vec::new();

        VarInt(chunk.x).write(&mut writer)?;
        VarInt(chunk.z).write(&mut writer)?;

        VarInt(dimension).write(&mut writer)?;
        let sub_chunk_count = chunk.section.count as u32;
        VarUInt(sub_chunk_count).write(&mut writer)?;
        // Optional sub-chunk request limit. Pumpkin sends complete chunks.
        false.write(&mut writer)?;
        cache_enabled.write(&mut writer)?;

        let mut blobs = Vec::new();

        let block_sections = chunk
            .section
            .block_sections
            .read()
            .map_err(|_| Error::other("block_sections read lock poisoned"))?;
        let min_y_section = (chunk.section.min_y >> 4) as i8;

        let mut subchunk_bytes_list = Vec::with_capacity(block_sections.len());

        for (i, block_palette) in block_sections.iter().enumerate() {
            let mut subchunk_buf = Vec::new();
            // Version 9: [version:byte][num_storages:byte][sub_chunk_index:byte]
            let y = (i as i8) + min_y_section;
            let num_storages = 1;
            subchunk_buf.write_all(&[VERSION, num_storages, y as u8])?;

            let network_repr = block_palette.convert_be_network();

            (network_repr.bits_per_entry << 1 | 1).write(&mut subchunk_buf)?;

            for data in network_repr.packed_data {
                data.write(&mut subchunk_buf)?;
            }

            match network_repr.palette {
                NetworkPalette::Single(id) => {
                    VarInt(i32::from(id)).write(&mut subchunk_buf)?;
                }
                NetworkPalette::Indirect(palette) => {
                    VarInt(palette.len() as i32).write(&mut subchunk_buf)?;
                    for id in palette {
                        VarInt(i32::from(id)).write(&mut subchunk_buf)?;
                    }
                }
                NetworkPalette::Direct => (),
            }

            subchunk_bytes_list.push(subchunk_buf);
        }

        let biome_sections = chunk
            .section
            .biome_sections
            .read()
            .map_err(|_| Error::other("biome_sections read lock poisoned"))?;

        let mut biome_buf = Vec::new();
        for biome_palette in biome_sections.iter() {
            let network_repr = biome_palette.convert_be_network();

            (network_repr.bits_per_entry << 1 | 1).write(&mut biome_buf)?;

            for data in network_repr.packed_data {
                data.write(&mut biome_buf)?;
            }

            match network_repr.palette {
                NetworkPalette::Single(id) => {
                    VarInt(i32::from(id)).write(&mut biome_buf)?;
                }
                NetworkPalette::Indirect(palette) => {
                    VarInt(palette.len() as i32).write(&mut biome_buf)?;
                    for id in palette {
                        VarInt(i32::from(id)).write(&mut biome_buf)?;
                    }
                }
                NetworkPalette::Direct => (),
            }
        }

        let block_actor_bytes = encode_block_actors(block_actors)?;

        if cache_enabled {
            for subchunk_buf in subchunk_bytes_list {
                let hash = xxh64(&subchunk_buf, 0);
                blobs.push((hash, subchunk_buf));
            }
            let biome_hash = xxh64(&biome_buf, 0);
            blobs.push((biome_hash, biome_buf));

            VarUInt(blobs.len() as u32).write(&mut writer)?;
            for (hash, _) in &blobs {
                writer.write_all(&hash.to_le_bytes())?;
            }

            // Palette data is cached, but the per-chunk border and block actor data is not.
            VarUInt(u32::try_from(1 + block_actor_bytes.len()).map_err(|_| {
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Bedrock block actor payload exceeds the packet size limit",
                )
            })?)
            .write(&mut writer)?;
            writer.write_all(&[0])?;
            writer.write_all(&block_actor_bytes)?;
        } else {
            VarUInt(0).write(&mut writer)?;

            let mut chunk_data = Vec::new();
            for subchunk_buf in subchunk_bytes_list {
                chunk_data.write_all(&subchunk_buf)?;
            }
            chunk_data.write_all(&biome_buf)?;
            chunk_data.write_all(&[0])?;
            chunk_data.write_all(&block_actor_bytes)?;

            VarUInt(chunk_data.len() as u32).write(&mut writer)?;
            writer.write_all(&chunk_data)?;
        }

        Ok((writer, blobs))
    }
}

impl PacketWrite for CLevelChunk<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let (encoded, _) = Self::encode_chunk(
            self.chunk,
            self.dimension,
            self.cache_enabled,
            self.block_actors,
        )?;
        writer.write_all(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pumpkin_nbt::{Nbt, compound::NbtCompound, deserializer::NbtReadHelperBedrock};
    use pumpkin_world::chunk::ChunkData;

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

    fn empty_chunk() -> ChunkData {
        ChunkData::empty(0, 0)
    }

    #[test]
    fn biomes_follow_subchunks_without_subchunk_headers() {
        let chunk = empty_chunk();
        let mut encoded = Vec::new();
        CLevelChunk {
            dimension: 0,
            cache_enabled: false,
            chunk: &chunk,
            block_actors: &[],
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

    #[test]
    fn block_actor_nbt_follows_the_chunk_border_data() {
        let chunk = empty_chunk();
        let mut block_actor = NbtCompound::new();
        block_actor.put_string("id", "Chest".to_string());
        block_actor.put_int("x", 1);
        block_actor.put_int("y", 64);
        block_actor.put_int("z", 2);

        let (encoded, _) = CLevelChunk::encode_chunk(&chunk, 0, true, &[block_actor]).unwrap();
        let mut offset = 0;
        for _ in 0..3 {
            read_var_uint(&encoded, &mut offset);
        }
        read_var_uint(&encoded, &mut offset);
        offset += 2;
        let blob_count = read_var_uint(&encoded, &mut offset) as usize;
        offset += blob_count * size_of::<u64>();
        let raw_len = read_var_uint(&encoded, &mut offset) as usize;
        let raw = &encoded[offset..offset + raw_len];

        assert_eq!(raw[0], 0);
        let mut reader = NbtReadHelperBedrock::new(Cursor::new(&raw[1..]));
        let parsed = Nbt::read(&mut reader).unwrap();
        assert_eq!(parsed.get_string("id"), Some("Chest"));
        assert_eq!(parsed.get_int("x"), Some(1));
    }
}
