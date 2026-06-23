//! Convert a parsed [`SlimeChunk`] into Pumpkin's native chunk representation.
//!
//! SlimeWorld stores chunk block states and biomes as the *vanilla* paletted
//! NBT (a palette of namespaced names, not Pumpkin's numeric ids), so the names
//! are resolved to global block-state / biome ids here before the section
//! palettes are built. Block entities and entities are mapped onto
//! [`ChunkData::pending_block_entities`] and [`ChunkEntityData`] respectively.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32};

use bytes::Bytes;
use pumpkin_data::{Block, biome::Biome, chunk::ChunkStatus};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use uuid::Uuid;

use crate::chunk::format::{ChunkSectionBiomes, ChunkSectionBlockStates, LightContainer};
use crate::chunk::palette::{BiomePalette, BlockPalette};
use crate::chunk::{ChunkData, ChunkEntityData, ChunkHeightmaps, ChunkLight, ChunkSections};
use crate::generation::section_coords;
use crate::tick::scheduler::ChunkTickScheduler;

use super::{LIGHT_ARRAY_SIZE, SlimeChunk, SlimeError};

/// Vanilla `block_states` section payload (`{ palette, data }`).
#[derive(Deserialize)]
struct VanillaBlockStates {
    palette: Vec<VanillaPaletteEntry>,
    #[serde(default)]
    data: Option<Box<[i64]>>,
}

/// A single vanilla block-state palette entry (`{ Name, Properties? }`).
#[derive(Deserialize)]
struct VanillaPaletteEntry {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties", default)]
    properties: Option<BTreeMap<String, String>>,
}

/// Vanilla `biomes` section payload (`{ palette, data }`).
#[derive(Deserialize)]
struct VanillaBiomes {
    palette: Vec<String>,
    #[serde(default)]
    data: Option<Box<[i64]>>,
}

/// A list of compound tags under a single key, e.g. `{ tileEntities: [ … ] }`.
#[derive(Deserialize)]
struct CompoundList {
    #[serde(alias = "tileEntities", alias = "entities", default)]
    items: Vec<NbtCompound>,
}

/// Strip the optional `minecraft:` namespace prefix.
fn strip_namespace(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// Resolve a vanilla palette entry (block name + properties) to a Pumpkin global
/// block-state id, falling back to air for unknown blocks.
fn palette_entry_to_state_id(entry: &VanillaPaletteEntry) -> u16 {
    let Some(block) = Block::from_registry_key(strip_namespace(&entry.name)) else {
        return Block::AIR.default_state.id;
    };
    match &entry.properties {
        Some(properties) if !properties.is_empty() => {
            let props: Vec<(&str, &str)> = properties
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            block.from_properties(&props).to_state_id(block)
        }
        _ => block.default_state.id,
    }
}

/// Resolve a vanilla biome name to a Pumpkin biome id, defaulting to plains.
fn biome_name_to_id(name: &str) -> u8 {
    Biome::from_name(strip_namespace(name)).map_or(Biome::PLAINS.id, |biome| biome.id)
}

/// Decode a vanilla UUID stored as an `int[4]` array (`[most_hi, most_lo, least_hi, least_lo]`).
fn uuid_from_int_array(array: &[i32]) -> Option<Uuid> {
    let [most_hi, most_lo, least_hi, least_lo] = *<&[i32; 4]>::try_from(array).ok()?;
    let most = (u64::from(most_hi.cast_unsigned()) << 32) | u64::from(most_lo.cast_unsigned());
    let least = (u64::from(least_hi.cast_unsigned()) << 32) | u64::from(least_lo.cast_unsigned());
    Some(Uuid::from_u128(
        (u128::from(most) << 64) | u128::from(least),
    ))
}

/// Parse a named-compound NBT segment into `T`; empty segments yield `None`.
fn parse_segment<T: for<'de> Deserialize<'de>>(bytes: &Bytes) -> Result<Option<T>, SlimeError> {
    if bytes.is_empty() {
        return Ok(None);
    }
    pumpkin_nbt::from_bytes::<T>(Cursor::new(bytes.as_ref()))
        .map(Some)
        .map_err(|_| SlimeError::Corrupt("invalid nbt segment"))
}

fn light_container(array: Option<&Bytes>) -> LightContainer {
    match array {
        Some(bytes) if bytes.len() == LIGHT_ARRAY_SIZE => {
            LightContainer::Full(bytes.to_vec().into_boxed_slice())
        }
        _ => LightContainer::Empty(0),
    }
}

/// Convert a [`SlimeChunk`] into Pumpkin's [`ChunkData`].
///
/// `min_section_y` is the bottom-most section index of the target dimension
/// (e.g. `-4` for the overworld); SlimeWorld stores sections bottom-up without
/// an explicit Y, so the absolute section Y of section `i` is `min_section_y + i`.
///
/// # Errors
///
/// Returns [`SlimeError`] if any of the chunk's NBT payloads are malformed.
pub(crate) fn chunk_to_chunk_data(
    chunk: &SlimeChunk,
    min_section_y: i32,
) -> Result<ChunkData, SlimeError> {
    let section_count = chunk.sections.len();
    let mut block_lights = vec![LightContainer::Empty(0); section_count];
    let mut sky_lights = vec![LightContainer::Empty(0); section_count];
    let mut block_palettes = vec![BlockPalette::default(); section_count];
    let mut biome_palettes = vec![BiomePalette::default(); section_count];

    for (index, section) in chunk.sections.iter().enumerate() {
        block_lights[index] = light_container(section.block_light.as_ref());
        sky_lights[index] = light_container(section.sky_light.as_ref());

        if let Some(states) = parse_segment::<VanillaBlockStates>(&section.block_states)? {
            let palette: Box<[u16]> = states
                .palette
                .iter()
                .map(palette_entry_to_state_id)
                .collect();
            block_palettes[index] = BlockPalette::from_disk_nbt(ChunkSectionBlockStates {
                data: states.data,
                palette,
            });
        }
        if let Some(biomes) = parse_segment::<VanillaBiomes>(&section.biomes)? {
            let palette: Box<[u8]> = biomes
                .palette
                .iter()
                .map(|name| biome_name_to_id(name))
                .collect();
            biome_palettes[index] = BiomePalette::from_disk_nbt(ChunkSectionBiomes {
                data: biomes.data,
                palette,
            });
        }
    }

    let light_engine = ChunkLight {
        block_light: block_lights.into_boxed_slice(),
        sky_light: sky_lights.into_boxed_slice(),
    };
    let has_light = chunk
        .sections
        .iter()
        .any(|section| section.block_light.is_some() || section.sky_light.is_some());

    let min_y = section_coords::section_to_block(min_section_y);
    let (random_tick_sections, randomly_ticking_mask) =
        ChunkSections::build_random_tick_sections_cache(&block_palettes);
    let section = ChunkSections {
        count: block_palettes.len(),
        block_sections: RwLock::new(block_palettes.into_boxed_slice()),
        random_tick_sections: RwLock::new(random_tick_sections),
        randomly_ticking_mask: AtomicU32::new(randomly_ticking_mask),
        biome_sections: RwLock::new(biome_palettes.into_boxed_slice()),
        min_y,
    };

    let heightmap =
        parse_segment::<ChunkHeightmaps>(&chunk.height_maps)?.unwrap_or(ChunkHeightmaps {
            world_surface: None,
            motion_blocking: None,
            motion_blocking_no_leaves: None,
        });

    let mut block_entities = FxHashMap::default();
    if let Some(list) = parse_segment::<CompoundList>(&chunk.block_entities)? {
        for entity in list.items {
            if let (Some(x), Some(y), Some(z)) = (
                entity.get_int("x"),
                entity.get_int("y"),
                entity.get_int("z"),
            ) {
                block_entities.insert(BlockPos::new(x, y, z), entity);
            }
        }
    }

    Ok(ChunkData {
        section,
        heightmap: std::sync::Mutex::new(heightmap),
        x: chunk.x,
        z: chunk.z,
        block_ticks: ChunkTickScheduler::default(),
        fluid_ticks: ChunkTickScheduler::default(),
        pending_block_entities: std::sync::Mutex::new(block_entities),
        light_engine: std::sync::Mutex::new(light_engine),
        light_populated: AtomicBool::new(has_light),
        status: ChunkStatus::Full,
        blending_data: None,
        // This chunk is freshly imported and must be persisted in Pumpkin's format.
        dirty: AtomicBool::new(true),
    })
}

/// Convert a [`SlimeChunk`]'s entities into a Pumpkin [`ChunkEntityData`].
///
/// # Errors
///
/// Returns [`SlimeError`] if the entity NBT payload is malformed.
pub(crate) fn chunk_to_entity_data(chunk: &SlimeChunk) -> Result<ChunkEntityData, SlimeError> {
    let mut data = FxHashMap::default();
    if let Some(list) = parse_segment::<CompoundList>(&chunk.entities)? {
        for entity in list.items {
            if let Some(uuid) = entity.get_int_array("UUID").and_then(uuid_from_int_array) {
                data.insert(uuid, entity);
            }
        }
    }

    Ok(ChunkEntityData {
        x: chunk.x,
        z: chunk.z,
        data: tokio::sync::Mutex::new(data),
        dirty: AtomicBool::new(true),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pumpkin_data::Block;
    use uuid::Uuid;

    use super::{
        VanillaPaletteEntry, biome_name_to_id, palette_entry_to_state_id, uuid_from_int_array,
    };

    #[test]
    fn resolves_air_palette_entry() {
        let entry = VanillaPaletteEntry {
            name: "minecraft:air".to_string(),
            properties: None,
        };
        assert_eq!(
            palette_entry_to_state_id(&entry),
            Block::AIR.default_state.id
        );
    }

    #[test]
    fn resolves_stone_palette_entry() {
        let entry = VanillaPaletteEntry {
            name: "minecraft:stone".to_string(),
            properties: None,
        };
        let expected = Block::from_registry_key("stone").unwrap().default_state.id;
        assert_eq!(palette_entry_to_state_id(&entry), expected);
    }

    #[test]
    fn resolves_block_with_properties() {
        // A property-bearing block should resolve to a *specific* (non-default) state.
        let mut properties = BTreeMap::new();
        properties.insert("facing".to_string(), "north".to_string());
        properties.insert("half".to_string(), "bottom".to_string());
        properties.insert("shape".to_string(), "straight".to_string());
        properties.insert("waterlogged".to_string(), "false".to_string());
        let entry = VanillaPaletteEntry {
            name: "minecraft:oak_stairs".to_string(),
            properties: Some(properties),
        };
        let block = Block::from_registry_key("oak_stairs").unwrap();
        let state_id = palette_entry_to_state_id(&entry);
        // The resolved state must belong to oak_stairs.
        assert!(block.states.iter().any(|state| state.id == state_id));
    }

    #[test]
    fn unknown_block_falls_back_to_air() {
        let entry = VanillaPaletteEntry {
            name: "modid:does_not_exist".to_string(),
            properties: None,
        };
        assert_eq!(
            palette_entry_to_state_id(&entry),
            Block::AIR.default_state.id
        );
    }

    #[test]
    fn resolves_known_biome() {
        assert_eq!(
            biome_name_to_id("minecraft:plains"),
            pumpkin_data::biome::Biome::PLAINS.id
        );
    }

    #[test]
    fn decodes_uuid_int_array() {
        let uuid = Uuid::from_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
        let bytes = uuid.as_u128();
        let array = [
            ((bytes >> 96) as u32).cast_signed(),
            ((bytes >> 64) as u32).cast_signed(),
            ((bytes >> 32) as u32).cast_signed(),
            (bytes as u32).cast_signed(),
        ];
        assert_eq!(uuid_from_int_array(&array), Some(uuid));
        assert_eq!(uuid_from_int_array(&[0, 0, 0]), None);
    }
}
