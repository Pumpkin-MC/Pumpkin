//! Serialize an in-memory world back into the `.slime` container format (v13).
//!
//! This is the inverse of [`super::reader`]: section block / biome palettes are
//! turned back into the vanilla string-paletted NBT, light is written as raw
//! nibble arrays, and the whole thing is framed and zstd-compressed exactly the
//! way AdvancedSlimePaper's `SlimeSerializer` does it.
//!
//! POI, block-tick and fluid-tick segments are not produced (the `flags` byte is
//! written as `0`); the world `extra` block (which carries `properties`) is
//! preserved verbatim from the source file.

use std::collections::BTreeMap;

use bytes::{BufMut, BytesMut};
use pumpkin_data::{Block, biome::Biome};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::{FxHashMap, FxHashSet};
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use serde::Serialize;

use crate::chunk::format::LightContainer;
use crate::chunk::palette::{BiomePalette, BlockPalette};
use crate::level::SyncChunk;

use super::{SLIME_MAGIC, SLIME_VERSION};

/// A minimal named, empty NBT compound (`TAG_Compound "" TAG_End`).
const EMPTY_COMPOUND: [u8; 4] = [0x0a, 0x00, 0x00, 0x00];

#[derive(Serialize)]
struct BlockStatesOut {
    palette: Vec<PaletteEntryOut>,
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    data: Option<Box<[i64]>>,
}

#[derive(Serialize)]
struct PaletteEntryOut {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties", skip_serializing_if = "Option::is_none")]
    properties: Option<BTreeMap<String, String>>,
}

#[derive(Serialize)]
struct BiomesOut {
    palette: Vec<String>,
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    data: Option<Box<[i64]>>,
}

#[derive(Serialize)]
struct TileEntitiesOut<'a> {
    #[serde(rename = "tileEntities")]
    tile_entities: Vec<&'a NbtCompound>,
}

#[derive(Serialize)]
struct EntitiesOut<'a> {
    entities: Vec<&'a NbtCompound>,
}

fn serialize_named<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    pumpkin_nbt::to_bytes(value, &mut buf).expect("serializing slime NBT segment should not fail");
    buf
}

/// Resolve a Pumpkin global block-state id back to a vanilla palette entry.
fn state_id_to_entry(state_id: u16) -> PaletteEntryOut {
    let block = Block::from_state_id(state_id);
    let properties = block.properties(state_id).and_then(|props| {
        let map: BTreeMap<String, String> = props
            .to_props()
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        (!map.is_empty()).then_some(map)
    });
    PaletteEntryOut {
        name: format!("minecraft:{}", block.name),
        properties,
    }
}

/// Resolve a Pumpkin biome id back to its namespaced name.
fn biome_id_to_name(biome_id: u8) -> String {
    Biome::from_id(biome_id).map_or_else(
        || "minecraft:plains".to_string(),
        |biome| format!("minecraft:{}", biome.registry_id),
    )
}

fn block_states_nbt(palette: &BlockPalette) -> Vec<u8> {
    let disk = palette.to_disk_nbt();
    serialize_named(&BlockStatesOut {
        palette: disk
            .palette
            .iter()
            .map(|&id| state_id_to_entry(id))
            .collect(),
        data: disk.data,
    })
}

fn biomes_nbt(palette: &BiomePalette) -> Vec<u8> {
    let disk = palette.to_disk_nbt();
    serialize_named(&BiomesOut {
        palette: disk
            .palette
            .iter()
            .map(|&id| biome_id_to_name(id))
            .collect(),
        data: disk.data,
    })
}

fn light_bytes(light: Option<&LightContainer>) -> Option<&[u8]> {
    match light {
        Some(LightContainer::Full(data)) => Some(data),
        _ => None,
    }
}

/// Write an `[i32 length][length bytes]` NBT segment.
fn put_segment(buf: &mut BytesMut, bytes: &[u8]) {
    buf.put_i32(i32::try_from(bytes.len()).unwrap_or(i32::MAX));
    buf.put_slice(bytes);
}

fn write_section(
    buf: &mut BytesMut,
    block: &BlockPalette,
    biome: &BiomePalette,
    block_light: Option<&LightContainer>,
    sky_light: Option<&LightContainer>,
) {
    let block_light = light_bytes(block_light);
    let sky_light = light_bytes(sky_light);

    let mut flags = 0u8;
    if block_light.is_some() {
        flags |= 1;
    }
    if sky_light.is_some() {
        flags |= 1 << 1;
    }
    buf.put_u8(flags);
    if let Some(data) = block_light {
        buf.put_slice(data);
    }
    if let Some(data) = sky_light {
        buf.put_slice(data);
    }
    put_segment(buf, &block_states_nbt(block));
    put_segment(buf, &biomes_nbt(biome));
}

fn write_chunk(buf: &mut BytesMut, chunk: Option<&SyncChunk>, entities: Option<&Vec<NbtCompound>>) {
    if let Some(chunk) = chunk {
        let block_sections = chunk
            .section
            .block_sections
            .read()
            .expect("block sections lock");
        let biome_sections = chunk
            .section
            .biome_sections
            .read()
            .expect("biome sections lock");
        let light = chunk.light_engine.lock().expect("light engine lock");

        let count = block_sections.len();
        buf.put_i32(i32::try_from(count).unwrap_or(i32::MAX));
        for index in 0..count {
            write_section(
                buf,
                &block_sections[index],
                &biome_sections[index],
                light.block_light.get(index),
                light.sky_light.get(index),
            );
        }
        drop(light);
        drop(biome_sections);
        drop(block_sections);

        let heightmaps = {
            let guard = chunk.heightmap.lock().expect("heightmap lock");
            serialize_named(&*guard)
        };
        put_segment(buf, &heightmaps);

        let tile_entities = {
            let guard = chunk
                .pending_block_entities
                .lock()
                .expect("block entities lock");
            serialize_named(&TileEntitiesOut {
                tile_entities: guard.values().collect(),
            })
        };
        put_segment(buf, &tile_entities);
    } else {
        buf.put_i32(0);
        put_segment(buf, &EMPTY_COMPOUND);
        put_segment(
            buf,
            &serialize_named(&TileEntitiesOut {
                tile_entities: Vec::new(),
            }),
        );
    }

    let entity_refs: Vec<&NbtCompound> = entities
        .map(|list| list.iter().collect())
        .unwrap_or_default();
    put_segment(
        buf,
        &serialize_named(&EntitiesOut {
            entities: entity_refs,
        }),
    );

    // Per-chunk persistent data container (unused): empty segment.
    buf.put_i32(0);
}

/// Serialize a whole world to the v13 `.slime` byte format.
///
/// `extra` is the (decompressed) world "extra" compound, preserved from the
/// source file; when empty an empty compound is written so the file stays valid.
pub(super) fn serialize_world(
    world_version: i32,
    extra: &[u8],
    chunks: &FxHashMap<Vector2<i32>, SyncChunk>,
    entities: &FxHashMap<Vector2<i32>, Vec<NbtCompound>>,
) -> Vec<u8> {
    let mut coords: FxHashSet<Vector2<i32>> = chunks.keys().copied().collect();
    coords.extend(entities.keys().copied());

    let mut body = BytesMut::new();
    body.put_i32(i32::try_from(coords.len()).unwrap_or(i32::MAX));
    for coord in &coords {
        body.put_i32(coord.x);
        body.put_i32(coord.y);
        write_chunk(&mut body, chunks.get(coord), entities.get(coord));
    }

    let mut file = BytesMut::new();
    file.put_slice(&SLIME_MAGIC);
    file.put_u8(SLIME_VERSION);
    file.put_i32(world_version);
    file.put_u8(0); // additionalWorldData flags: none written back
    write_compressed(&mut file, &body);
    let extra = if extra.is_empty() {
        &EMPTY_COMPOUND
    } else {
        extra
    };
    write_compressed(&mut file, extra);
    file.to_vec()
}

fn write_compressed(file: &mut BytesMut, data: &[u8]) {
    let compressed = compress_to_vec(data, CompressionLevel::Fastest);
    file.put_i32(i32::try_from(compressed.len()).unwrap_or(i32::MAX));
    file.put_i32(i32::try_from(data.len()).unwrap_or(i32::MAX));
    file.put_slice(&compressed);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use pumpkin_data::Block;
    use pumpkin_util::math::vector2::Vector2;
    use rustc_hash::FxHashMap;

    use super::super::SlimeChunk;
    use super::super::model::SlimeSection;
    use super::super::{chunk_to_chunk_data, read_slime_world};
    use super::{BlockStatesOut, PaletteEntryOut, serialize_named, serialize_world};

    fn stone_block_states() -> Bytes {
        Bytes::from(serialize_named(&BlockStatesOut {
            palette: vec![PaletteEntryOut {
                name: "minecraft:stone".to_string(),
                properties: None,
            }],
            data: None,
        }))
    }

    #[test]
    fn round_trips_a_stone_chunk() {
        let stone_id = Block::from_registry_key("stone").unwrap().default_state.id;

        // Build a one-section chunk holding only stone, via the read-side converter.
        let slime_chunk = SlimeChunk {
            x: 4,
            z: -2,
            sections: vec![SlimeSection {
                block_light: None,
                sky_light: None,
                block_states: stone_block_states(),
                biomes: Bytes::new(),
            }],
            height_maps: Bytes::new(),
            block_entities: Bytes::new(),
            entities: Bytes::new(),
            block_ticks: Bytes::new(),
            fluid_ticks: Bytes::new(),
            poi: Bytes::new(),
        };
        let chunk_data = chunk_to_chunk_data(&slime_chunk, -4).expect("convert");

        // Serialize the whole world, then read it back.
        let mut chunks = FxHashMap::default();
        chunks.insert(Vector2::new(4, -2), Arc::new(chunk_data));
        let bytes = serialize_world(3700, &[], &chunks, &FxHashMap::default());

        let world = read_slime_world(Bytes::from(bytes)).expect("read back");
        assert_eq!(world.world_version, 3700);
        assert_eq!(world.chunks.len(), 1);
        let chunk = &world.chunks[0];
        assert_eq!((chunk.x, chunk.z), (4, -2));

        // The single section must still resolve to stone.
        let restored = chunk_to_chunk_data(chunk, -4).expect("convert back");
        let sections = restored.section.block_sections.read().unwrap();
        assert_eq!(
            sections[0].to_disk_nbt().palette.first().copied(),
            Some(stone_id)
        );
    }
}
