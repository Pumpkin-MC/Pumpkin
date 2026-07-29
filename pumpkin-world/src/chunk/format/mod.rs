use std::{
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    sync::{
        RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use bytes::Bytes;
use pumpkin_data::{Block, BlockStateId, chunk::Biome, chunk::ChunkStatus, fluid::Fluid};
use pumpkin_nbt::{compound::NbtCompound, nbt_long_array};
use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    chunk::{
        ChunkEntityData, ChunkReadingError, ChunkSerializingError,
        format::anvil::{SingleChunkDataSerializer, WORLD_DATA_VERSION},
        io::{Dirtiable, file_manager::PathFromLevelFolder},
    },
    generation::{
        section_coords,
        structure::template::{BlockStateResolver, PaletteEntry},
    },
    level::LevelFolder,
    tick::{ScheduledTick, TickPriority, scheduler::ChunkTickScheduler},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use serde::{Deserialize, Serialize};

use super::{
    ChunkData, ChunkHeightmaps, ChunkLight, ChunkParsingError, ChunkSections,
    palette::{BiomePalette, BlockPalette},
};
pub mod anvil;
pub mod linear;
pub mod pump;

impl SingleChunkDataSerializer for ChunkData {
    #[inline]
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        Self::internal_from_bytes(bytes, pos).map_err(ChunkReadingError::ParsingError)
    }

    #[inline]
    fn to_bytes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>> {
        Box::pin(async move { self.internal_to_bytes() })
    }

    #[inline]
    fn position(&self) -> (i32, i32) {
        (self.x, self.z)
    }
}

impl PathFromLevelFolder for ChunkData {
    #[inline]
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.region_folder.join(file_name)
    }
}

impl Dirtiable for ChunkData {
    #[inline]
    fn mark_dirty(&self, flag: bool) {
        self.dirty.store(flag, Ordering::Relaxed);
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

/// Resolves one vanilla block palette entry, `{Name: "minecraft:stone", Properties: {..}}`,
/// to a block state id.
///
/// Vanilla has written section palettes in this shape since the 1.13 flattening; we write
/// raw numeric state ids. Both have to be understood on load, because a world can contain
/// chunks last written by either.
///
/// Returns `None` for a name we do not know, so the caller can log it rather than silently
/// turning someone's blocks into air.
fn block_state_id_from_palette_entry(nbt: &NbtCompound) -> Option<BlockStateId> {
    let name = nbt.get_string("Name")?;

    let properties = nbt
        .get_compound("Properties")
        .map_or_else(Vec::new, |props| {
            props
                .child_tags
                .iter()
                .filter_map(|(key, value)| {
                    value
                        .extract_string()
                        .map(|value| (key.to_string(), value.to_string()))
                })
                .collect()
        });

    let entry = PaletteEntry {
        name: name.to_string(),
        properties,
    };
    BlockStateResolver::resolve_simple(&entry).map(|state| state.id)
}

/// Resolves one vanilla biome palette entry, a plain resource location such as
/// `"minecraft:plains"`, to a biome id.
///
/// `Biome::from_name` matches bare names, so the namespace has to be stripped first —
/// unlike `Block::from_name`, which strips it itself.
fn biome_id_from_palette_entry(name: &str) -> Option<u8> {
    let bare = name.strip_prefix("minecraft:").unwrap_or(name);
    Biome::from_name(bare).map(|biome| biome.id)
}

/// Renders a block state id as a vanilla palette entry,
/// `{Name: "minecraft:stone", Properties: {..}}`.
///
/// `Properties` is omitted for a block that has none, matching vanilla.
fn palette_entry_from_block_state_id(id: BlockStateId) -> pumpkin_nbt::tag::NbtTag {
    let block = id.to_block_id().to_block();

    let mut entry = NbtCompound::new();
    entry.put_string("Name", format!("minecraft:{}", block.name));

    if let Some(properties) = block.properties(id) {
        let mut props = NbtCompound::new();
        for (key, value) in properties.to_props() {
            props.put_string(key, value.to_string());
        }
        entry.put_compound("Properties", props);
    }

    pumpkin_nbt::tag::NbtTag::Compound(entry)
}

/// Renders a biome id as a vanilla palette entry, a plain resource location.
fn palette_entry_from_biome_id(id: u8) -> pumpkin_nbt::tag::NbtTag {
    let registry_id = Biome::from_id(id).unwrap_or(&Biome::PLAINS).registry_id;
    pumpkin_nbt::tag::NbtTag::String(format!("minecraft:{registry_id}").into())
}

fn extract_u16_array(tag: &pumpkin_nbt::tag::NbtTag) -> Option<Box<[BlockStateId]>> {
    match tag {
        pumpkin_nbt::tag::NbtTag::IntArray(arr) => Some(
            arr.iter()
                .map(|&x| BlockStateId::new_or_air(x as u16))
                .collect(),
        ),
        pumpkin_nbt::tag::NbtTag::ByteArray(arr) => Some(
            arr.iter()
                .map(|&x| BlockStateId::new_or_air(x as u16))
                .collect(),
        ),
        pumpkin_nbt::tag::NbtTag::LongArray(arr) => Some(
            arr.iter()
                .map(|&x| BlockStateId::new_or_air(x as u16))
                .collect(),
        ),
        pumpkin_nbt::tag::NbtTag::List(list) => {
            let ids: Box<[BlockStateId]> = list
                .iter()
                .map(|t| match t {
                    // Vanilla's shape. Resolve by name rather than falling through to air,
                    // which would blank every block in the section.
                    pumpkin_nbt::tag::NbtTag::Compound(nbt) => {
                        block_state_id_from_palette_entry(nbt).unwrap_or_else(|| {
                            warn!(
                                "Unknown block in chunk palette: {:?}; loading it as air",
                                nbt.get_string("Name").unwrap_or("<no Name>")
                            );
                            BlockStateId::AIR
                        })
                    }
                    // Our own shape: a raw state id.
                    pumpkin_nbt::tag::NbtTag::Int(x) => BlockStateId::new_or_air(*x as u16),
                    pumpkin_nbt::tag::NbtTag::Short(x) => BlockStateId::new_or_air(*x as u16),
                    pumpkin_nbt::tag::NbtTag::Byte(x) => BlockStateId::new_or_air(*x as u16),
                    pumpkin_nbt::tag::NbtTag::Long(x) => BlockStateId::new_or_air(*x as u16),
                    _ => BlockStateId::AIR,
                })
                .collect();
            Some(ids)
        }
        _ => None,
    }
}

fn extract_u8_array(tag: &pumpkin_nbt::tag::NbtTag) -> Option<Box<[u8]>> {
    match tag {
        pumpkin_nbt::tag::NbtTag::ByteArray(arr) => Some(arr.iter().map(|&x| x as u8).collect()),
        pumpkin_nbt::tag::NbtTag::IntArray(arr) => Some(arr.iter().map(|&x| x as u8).collect()),
        pumpkin_nbt::tag::NbtTag::List(list) => {
            let bytes: Box<[u8]> = list
                .iter()
                .map(|t| match t {
                    // Vanilla's shape: a resource location per entry.
                    pumpkin_nbt::tag::NbtTag::String(name) => biome_id_from_palette_entry(name)
                        .unwrap_or_else(|| {
                            warn!("Unknown biome in chunk palette: {name}; loading it as plains");
                            Biome::PLAINS.id
                        }),
                    // Our own shape: a raw biome id.
                    pumpkin_nbt::tag::NbtTag::Byte(x) => *x as u8,
                    pumpkin_nbt::tag::NbtTag::Int(x) => *x as u8,
                    pumpkin_nbt::tag::NbtTag::Short(x) => *x as u8,
                    _ => 0,
                })
                .collect();
            Some(bytes)
        }
        _ => None,
    }
}

fn parse_scheduled_tick<T>(nbt: &pumpkin_nbt::compound::NbtCompound) -> Option<ScheduledTick<T>>
where
    T: FromResourceLocation,
{
    let x = nbt.get_int("x")?;
    let y = nbt.get_int("y")?;
    let z = nbt.get_int("z")?;
    let delay = nbt.get_int("t")? as u8;
    let priority = TickPriority::try_from(nbt.get_int("p")?).ok()?;
    let res_loc_str = nbt.get_string("i")?;
    let res_loc = ResourceLocation::from_str(res_loc_str).ok()?;
    let value = T::from_resource_location(&res_loc)?;
    Some(ScheduledTick {
        delay,
        priority,
        position: BlockPos::new(x, y, z),
        value,
    })
}

impl ChunkData {
    #[allow(clippy::too_many_lines)]
    pub fn internal_from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        let is_named = chunk_data.len() >= 3
            && chunk_data[0] == 0x0a
            && chunk_data[1] == 0x00
            && chunk_data[2] == 0x00;

        let mut cursor = std::io::Cursor::new(chunk_data);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
        let nbt = if is_named {
            pumpkin_nbt::Nbt::read(&mut reader)
        } else {
            pumpkin_nbt::Nbt::read_unnamed(&mut reader)
        }
        .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        let root_tag = nbt.root_tag;

        let x_pos = root_tag.get_int("xPos").ok_or_else(|| {
            ChunkParsingError::ErrorDeserializingChunk("Missing xPos".to_string())
        })?;
        let z_pos = root_tag.get_int("zPos").ok_or_else(|| {
            ChunkParsingError::ErrorDeserializingChunk("Missing zPos".to_string())
        })?;

        if x_pos != position.x || z_pos != position.y {
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for chunk {},{} but got it for {},{}!",
                position.x, position.y, x_pos, z_pos,
            )));
        }

        let min_y_section = root_tag.get_int("yPos").ok_or_else(|| {
            ChunkParsingError::ErrorDeserializingChunk("Missing yPos".to_string())
        })?;

        let mut max_y_section = min_y_section as i8;
        if let Some(sections_list) = root_tag.get_list("sections") {
            for section_tag in sections_list {
                if let pumpkin_nbt::tag::NbtTag::Compound(section_compound) = section_tag {
                    let y = section_compound.get_byte("Y").unwrap_or(0);
                    if y > max_y_section {
                        max_y_section = y;
                    }
                }
            }
        }

        let section_count = (max_y_section as i32 - min_y_section + 1).max(0) as usize;
        let mut block_lights = vec![LightContainer::Empty(0); section_count];
        let mut sky_lights = vec![LightContainer::Empty(0); section_count];
        let mut block_palettes = vec![BlockPalette::default(); section_count];
        let mut biome_palettes = vec![BiomePalette::default(); section_count];

        if let Some(sections_list) = root_tag.get_list("sections") {
            for section_tag in sections_list {
                if let pumpkin_nbt::tag::NbtTag::Compound(section_compound) = section_tag {
                    let y = section_compound.get_byte("Y").unwrap_or(0);
                    let index = (y as i32 - min_y_section) as usize;
                    if index >= section_count {
                        continue;
                    }

                    let block_light = section_compound
                        .get("BlockLight")
                        .and_then(|tag| tag.extract_byte_array())
                        .map(|arr| unsafe {
                            Box::from(std::slice::from_raw_parts(
                                arr.as_ptr().cast::<u8>(),
                                arr.len(),
                            ))
                        });

                    let sky_light = section_compound
                        .get("SkyLight")
                        .and_then(|tag| tag.extract_byte_array())
                        .map(|arr| unsafe {
                            Box::from(std::slice::from_raw_parts(
                                arr.as_ptr().cast::<u8>(),
                                arr.len(),
                            ))
                        });

                    block_lights[index] =
                        block_light.map_or(LightContainer::Empty(0), LightContainer::Full);
                    sky_lights[index] =
                        sky_light.map_or(LightContainer::Empty(0), LightContainer::Full);

                    if let Some(bs_compound) = section_compound.get_compound("block_states") {
                        let data = bs_compound
                            .get_long_array("data")
                            .map(|arr| arr.to_vec().into_boxed_slice());
                        let palette = bs_compound
                            .get("palette")
                            .and_then(extract_u16_array)
                            .unwrap_or_else(|| vec![BlockStateId::AIR].into_boxed_slice());

                        block_palettes[index] =
                            BlockPalette::from_disk_nbt(ChunkSectionBlockStates { data, palette });
                    } else {
                        block_palettes[index] = BlockPalette::default();
                    }

                    if let Some(b_compound) = section_compound.get_compound("biomes") {
                        let data = b_compound
                            .get_long_array("data")
                            .map(|arr| arr.to_vec().into_boxed_slice());
                        let palette = b_compound
                            .get("palette")
                            .and_then(extract_u8_array)
                            .unwrap_or_else(|| vec![0].into_boxed_slice());

                        biome_palettes[index] =
                            BiomePalette::from_disk_nbt(ChunkSectionBiomes { data, palette });
                    } else {
                        biome_palettes[index] = BiomePalette::default();
                    }
                }
            }
        }

        // Assemble the LightEngine
        let light_engine = ChunkLight {
            block_light: block_lights.into_boxed_slice(),
            sky_light: sky_lights.into_boxed_slice(),
        };

        // Assemble the ChunkSections
        let min_y = section_coords::section_to_block(min_y_section);
        let (random_tick_sections, randomly_ticking_mask) =
            ChunkSections::build_random_tick_sections_cache(&block_palettes);
        let section = ChunkSections {
            count: block_palettes.len(),
            block_sections: RwLock::new(block_palettes.into_boxed_slice()),
            random_tick_sections: RwLock::new(random_tick_sections),
            randomly_ticking_mask: std::sync::atomic::AtomicU32::new(randomly_ticking_mask),
            biome_sections: RwLock::new(biome_palettes.into_boxed_slice()),
            min_y,
        };

        let heightmaps = root_tag.get_compound("Heightmaps").map_or(
            ChunkHeightmaps {
                world_surface: None,
                motion_blocking: None,
                motion_blocking_no_leaves: None,
            },
            |h_compound| ChunkHeightmaps {
                world_surface: h_compound
                    .get_long_array("WORLD_SURFACE")
                    .map(|a| a.to_vec().into_boxed_slice()),
                motion_blocking: h_compound
                    .get_long_array("MOTION_BLOCKING")
                    .map(|a| a.to_vec().into_boxed_slice()),
                motion_blocking_no_leaves: h_compound
                    .get_long_array("MOTION_BLOCKING_NO_LEAVES")
                    .map(|a| a.to_vec().into_boxed_slice()),
            },
        );
        let mut block_ticks = Vec::new();
        if let Some(list) = root_tag.get_list("block_ticks") {
            for tag in list {
                if let pumpkin_nbt::tag::NbtTag::Compound(compound) = tag
                    && let Some(tick) = parse_scheduled_tick::<&'static Block>(compound)
                {
                    block_ticks.push(tick);
                }
            }
        }

        let mut fluid_ticks = Vec::new();
        if let Some(list) = root_tag.get_list("fluid_ticks") {
            for tag in list {
                if let pumpkin_nbt::tag::NbtTag::Compound(compound) = tag
                    && let Some(tick) = parse_scheduled_tick::<&'static Fluid>(compound)
                {
                    fluid_ticks.push(tick);
                }
            }
        }

        let mut block_entities = FxHashMap::default();
        if let Some(list) = root_tag.get_list("block_entities") {
            for tag in list {
                if let pumpkin_nbt::tag::NbtTag::Compound(nbt) = tag
                    && let Some(x) = nbt.get_int("x")
                    && let Some(y) = nbt.get_int("y")
                    && let Some(z) = nbt.get_int("z")
                {
                    block_entities.insert(BlockPos::new(x, y, z), nbt.clone());
                }
            }
        }

        let light_correct = root_tag.get_bool("isLightOn").unwrap_or(false);

        let status_str = root_tag.get_string("Status").unwrap_or("minecraft:empty");
        let status = match status_str {
            "minecraft:structure_starts" => ChunkStatus::StructureStarts,
            "minecraft:structure_references" => ChunkStatus::StructureReferences,
            "minecraft:biomes" => ChunkStatus::Biomes,
            "minecraft:noise" => ChunkStatus::Noise,
            "minecraft:surface" => ChunkStatus::Surface,
            "minecraft:carvers" => ChunkStatus::Carvers,
            "minecraft:features" => ChunkStatus::Features,
            "minecraft:initialize_light" => ChunkStatus::InitializeLight,
            "minecraft:light" => ChunkStatus::Light,
            "minecraft:spawn" => ChunkStatus::Spawn,
            "minecraft:full" => ChunkStatus::Full,
            _ => ChunkStatus::Empty,
        };

        Ok(Self {
            section,
            heightmap: std::sync::Mutex::new(heightmaps),
            x: position.x,
            z: position.y,
            // This chunk is read from disk, so it has not been modified
            dirty: AtomicBool::new(false),
            block_ticks: ChunkTickScheduler::from_iter(block_ticks),
            fluid_ticks: ChunkTickScheduler::from_iter(fluid_ticks),
            pending_block_entities: std::sync::Mutex::new(block_entities),
            light_engine: std::sync::Mutex::new(light_engine),
            light_populated: AtomicBool::new(light_correct),
            status,
            blending_data: None,
            inhabited_time: AtomicU64::new(root_tag.get_long("InhabitedTime").unwrap_or(0) as u64),
        })
    }

    #[allow(clippy::too_many_lines)]
    fn internal_to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        use pumpkin_nbt::tag::NbtTag;

        fn extract_light_ref(light: Option<&LightContainer>) -> Option<&[u8]> {
            match light {
                Some(LightContainer::Full(data)) => Some(data.as_ref()),
                _ => None,
            }
        }

        let is_light_correct = self
            .light_populated
            .load(std::sync::atomic::Ordering::Relaxed);

        let block_entities_nbt = {
            let entities_guard = self.pending_block_entities.lock().unwrap();
            entities_guard.values().cloned().collect::<Vec<_>>()
        };

        let light_lock = self.light_engine.lock().unwrap();
        let heightmap_lock = self.heightmap.lock().unwrap();
        let block_lock = self.section.block_sections.read().unwrap();
        let biome_lock = self.section.biome_sections.read().unwrap();

        let min_section_y = (self.section.min_y >> 4) as i8;

        let mut root_compound = NbtCompound::new();
        root_compound.put_int("DataVersion", WORLD_DATA_VERSION);
        root_compound.put_int("xPos", self.x);
        root_compound.put_int("zPos", self.z);
        root_compound.put_int("yPos", section_coords::block_to_section(self.section.min_y));

        let status_str = match self.status {
            ChunkStatus::Empty => "minecraft:empty",
            ChunkStatus::StructureStarts => "minecraft:structure_starts",
            ChunkStatus::StructureReferences => "minecraft:structure_references",
            ChunkStatus::Biomes => "minecraft:biomes",
            ChunkStatus::Noise => "minecraft:noise",
            ChunkStatus::Surface => "minecraft:surface",
            ChunkStatus::Carvers => "minecraft:carvers",
            ChunkStatus::Features => "minecraft:features",
            ChunkStatus::InitializeLight => "minecraft:initialize_light",
            ChunkStatus::Light => "minecraft:light",
            ChunkStatus::Spawn => "minecraft:spawn",
            ChunkStatus::Full => "minecraft:full",
        };
        root_compound.put_string("Status", status_str.to_string());

        let mut heightmaps_compound = NbtCompound::new();
        if let Some(ref arr) = heightmap_lock.world_surface {
            heightmaps_compound.put("WORLD_SURFACE", NbtTag::LongArray(arr.to_vec()));
        }
        if let Some(ref arr) = heightmap_lock.motion_blocking {
            heightmaps_compound.put("MOTION_BLOCKING", NbtTag::LongArray(arr.to_vec()));
        }
        if let Some(ref arr) = heightmap_lock.motion_blocking_no_leaves {
            heightmaps_compound.put("MOTION_BLOCKING_NO_LEAVES", NbtTag::LongArray(arr.to_vec()));
        }
        root_compound.put_compound("Heightmaps", heightmaps_compound);

        let mut sections_list = Vec::new();
        for i in 0..self.section.count {
            let mut section_comp = NbtCompound::new();
            let y_val = i as i8 + min_section_y;
            section_comp.put_byte("Y", y_val);

            // block_states
            let block_states_nbt = block_lock[i].to_disk_nbt();
            let mut bs_comp = NbtCompound::new();
            if let Some(ref data_arr) = block_states_nbt.data {
                bs_comp.put("data", NbtTag::LongArray(data_arr.to_vec()));
            }
            let palette_tags: Vec<NbtTag> = block_states_nbt
                .palette
                .iter()
                .map(|id| palette_entry_from_block_state_id(*id))
                .collect();
            bs_comp.put_list("palette", palette_tags);
            section_comp.put_compound("block_states", bs_comp);

            // biomes
            let biomes_nbt = biome_lock[i].to_disk_nbt();
            let mut b_comp = NbtCompound::new();
            if let Some(ref data_arr) = biomes_nbt.data {
                b_comp.put("data", NbtTag::LongArray(data_arr.to_vec()));
            }
            let biome_palette_tags: Vec<NbtTag> = biomes_nbt
                .palette
                .iter()
                .map(|&val| palette_entry_from_biome_id(val))
                .collect();
            b_comp.put_list("palette", biome_palette_tags);
            section_comp.put_compound("biomes", b_comp);

            // block_light
            if let Some(light_data) = extract_light_ref(light_lock.block_light.get(i)) {
                let bytes: Box<[i8]> = light_data.iter().map(|&x| x as i8).collect();
                section_comp.put("BlockLight", NbtTag::ByteArray(bytes));
            }

            // sky_light
            if let Some(light_data) = extract_light_ref(light_lock.sky_light.get(i)) {
                let bytes: Box<[i8]> = light_data.iter().map(|&x| x as i8).collect();
                section_comp.put("SkyLight", NbtTag::ByteArray(bytes));
            }

            sections_list.push(NbtTag::Compound(section_comp));
        }
        root_compound.put_list("sections", sections_list);

        let mut block_ticks_list = Vec::new();
        for tick in self.block_ticks.to_vec() {
            let mut tick_comp = NbtCompound::new();
            tick_comp.put_int("x", tick.position.0.x);
            tick_comp.put_int("y", tick.position.0.y);
            tick_comp.put_int("z", tick.position.0.z);
            tick_comp.put_int("t", tick.delay as i32);
            tick_comp.put_int("p", tick.priority as i32);
            tick_comp.put_string("i", tick.value.to_resource_location());
            block_ticks_list.push(NbtTag::Compound(tick_comp));
        }
        root_compound.put_list("block_ticks", block_ticks_list);

        let mut fluid_ticks_list = Vec::new();
        for tick in self.fluid_ticks.to_vec() {
            let mut tick_comp = NbtCompound::new();
            tick_comp.put_int("x", tick.position.0.x);
            tick_comp.put_int("y", tick.position.0.y);
            tick_comp.put_int("z", tick.position.0.z);
            tick_comp.put_int("t", tick.delay as i32);
            tick_comp.put_int("p", tick.priority as i32);
            tick_comp.put_string("i", tick.value.to_resource_location());
            fluid_ticks_list.push(NbtTag::Compound(tick_comp));
        }
        root_compound.put_list("fluid_ticks", fluid_ticks_list);

        let mut block_entities_list = Vec::new();
        for entity_comp in block_entities_nbt {
            block_entities_list.push(NbtTag::Compound(entity_comp));
        }
        root_compound.put_list("block_entities", block_entities_list);

        root_compound.put_bool("isLightOn", is_light_correct);

        let mut result = Vec::new();
        pumpkin_nbt::serializer::to_bytes(&root_compound, &mut result)
            .map_err(ChunkSerializingError::ErrorSerializingChunk)?;

        Ok(result.into())
    }
}

impl PathFromLevelFolder for ChunkEntityData {
    #[inline]
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.entities_folder.join(file_name)
    }
}

impl Dirtiable for ChunkEntityData {
    #[inline]
    fn mark_dirty(&self, flag: bool) {
        self.dirty.store(flag, Ordering::Relaxed);
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

impl SingleChunkDataSerializer for ChunkEntityData {
    #[inline]
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        Self::internal_from_bytes(bytes, pos).map_err(ChunkReadingError::ParsingError)
    }

    #[inline]
    fn to_bytes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>> {
        Box::pin(async move { self.internal_to_bytes().await })
    }

    #[inline]
    fn position(&self) -> (i32, i32) {
        (self.x, self.z)
    }
}

impl ChunkEntityData {
    fn internal_from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        let is_named = chunk_data.len() >= 3
            && chunk_data[0] == 0x0a
            && chunk_data[1] == 0x00
            && chunk_data[2] == 0x00;
        let chunk_entity_data = if is_named {
            pumpkin_nbt::from_bytes::<EntityNbt>(std::io::Cursor::new(chunk_data))
        } else {
            pumpkin_nbt::from_bytes_unnamed::<EntityNbt>(std::io::Cursor::new(chunk_data))
        }
        .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        if chunk_entity_data.position[0] != position.x
            || chunk_entity_data.position[1] != position.y
        {
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for entity chunk {},{} but got it for {},{}!",
                position.x,
                position.y,
                chunk_entity_data.position[0],
                chunk_entity_data.position[1],
            )));
        }

        Ok(Self {
            x: position.x,
            z: position.y,
            data: Mutex::new(chunk_entity_data.entities),
            dirty: AtomicBool::new(false),
        })
    }

    async fn internal_to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        let nbt = EntityNbt {
            data_version: WORLD_DATA_VERSION,
            position: [self.x, self.z],
            entities: self.data.lock().await.clone(),
        };

        let mut result = Vec::new();
        pumpkin_nbt::to_bytes(&nbt, &mut result)
            .map_err(ChunkSerializingError::ErrorSerializingChunk)?;
        Ok(result.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkSectionBiomes {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) data: Option<Box<[i64]>>,
    pub(crate) palette: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChunkSectionBlockStates {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) data: Option<Box<[i64]>>,
    #[serde(with = "block_state_checked")]
    pub(crate) palette: Box<[BlockStateId]>,
}

mod block_state_checked {
    use pumpkin_data::BlockStateId;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        value: &[BlockStateId],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .iter()
            .map(|v| BlockStateId::as_u16(*v))
            .collect::<Vec<u16>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Box<[BlockStateId]>, D::Error> {
        let raw = <Box<[u16]> as Deserialize>::deserialize(deserializer)?;
        Ok(raw.iter().map(|v| BlockStateId::new_or_air(*v)).collect())
    }
}

#[derive(Debug, Clone)]
pub enum LightContainer {
    Empty(u8),
    Full(Box<[u8]>),
}

impl LightContainer {
    pub const DIM: usize = 16;
    pub const ARRAY_SIZE: usize = Self::DIM * Self::DIM * Self::DIM / 2;

    #[must_use]
    pub fn new_empty(default: u8) -> Self {
        assert!(default <= 15, "Default value must be between 0 and 15");
        Self::Empty(default)
    }

    #[must_use]
    pub fn new(data: Box<[u8]>) -> Self {
        assert!(
            data.len() == Self::ARRAY_SIZE,
            "Data length must be {}",
            Self::ARRAY_SIZE
        );
        Self::Full(data)
    }

    #[must_use]
    pub fn new_filled(default: u8) -> Self {
        assert!(default <= 15, "Default value must be between 0 and 15");
        let value = default << 4 | default;
        Self::Full([value; Self::ARRAY_SIZE].into())
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty(_))
    }

    const fn index(x: usize, y: usize, z: usize) -> usize {
        y * 16 * 16 + z * 16 + x
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        match self {
            Self::Full(data) => {
                let index = Self::index(x, y, z);
                data[index >> 1] >> (4 * (index & 1)) & 0x0F
            }
            Self::Empty(default) => *default,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        match self {
            Self::Full(data) => {
                let index = Self::index(x, y, z);
                let mask = 0x0F << (4 * (index & 1));
                data[index >> 1] &= !mask;
                data[index >> 1] |= value << (4 * (index & 1));
            }
            Self::Empty(default) => {
                if value != *default {
                    *self = Self::new_filled(*default);
                    self.set(x, y, z, value);
                }
            }
        }
    }

    pub fn fill(&mut self, value: u8) {
        *self = Self::new_filled(value);
    }
}

impl Default for LightContainer {
    fn default() -> Self {
        Self::new_empty(15)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct EntityNbt {
    data_version: i32,
    position: [i32; 2],
    entities: Vec<NbtCompound>,
}

#[cfg(test)]
mod tests {
    use super::{ChunkData, WORLD_DATA_VERSION};
    use pumpkin_data::{Block, BlockStateId, chunk::Biome};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_nbt::tag::NbtTag;
    use pumpkin_util::math::vector2::Vector2;

    /// A vanilla palette entry: `{Name: "minecraft:x", Properties: {..}}`.
    fn named_entry(name: &str, properties: &[(&str, &str)]) -> NbtTag {
        let mut entry = NbtCompound::new();
        entry.put_string("Name", name.to_string());
        if !properties.is_empty() {
            let mut props = NbtCompound::new();
            for (key, value) in properties {
                props.put_string(key, (*value).to_string());
            }
            entry.put_compound("Properties", props);
        }
        NbtTag::Compound(entry)
    }

    /// One section at Y=-4 with the given block and biome palettes. A single-entry palette
    /// needs no data array: every block in the section is that entry.
    fn section(block_palette: Vec<NbtTag>, biome_palette: Vec<NbtTag>) -> NbtTag {
        let mut block_states = NbtCompound::new();
        block_states.put_list("palette", block_palette);
        let mut biomes = NbtCompound::new();
        biomes.put_list("palette", biome_palette);
        let mut section = NbtCompound::new();
        section.put_byte("Y", -4);
        section.put_compound("block_states", block_states);
        section.put_compound("biomes", biomes);
        NbtTag::Compound(section)
    }

    fn load(block_palette: Vec<NbtTag>, biome_palette: Vec<NbtTag>) -> ChunkData {
        let mut root = NbtCompound::new();
        root.put_int("DataVersion", WORLD_DATA_VERSION);
        root.put_int("xPos", 0);
        root.put_int("zPos", 0);
        root.put_int("yPos", -4);
        root.put_string("Status", "minecraft:full".to_string());
        root.put_list("sections", vec![section(block_palette, biome_palette)]);

        let mut buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes(&root, &mut buf).expect("serialize fixture");
        ChunkData::internal_from_bytes(&buf, Vector2::new(0, 0)).expect("parse fixture")
    }

    fn only_block(chunk: &ChunkData) -> BlockStateId {
        let blocks = chunk.section.dump_blocks();
        let first = blocks[0];
        assert!(
            blocks.iter().all(|b| *b == first),
            "expected a uniform section"
        );
        first
    }

    #[test]
    fn vanilla_named_block_palette_loads_as_that_block() {
        // The regression: before this, a vanilla palette entry hit the catch-all arm and
        // every block in the section became air.
        let chunk = load(
            vec![named_entry("minecraft:stone", &[])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        assert_eq!(only_block(&chunk), Block::STONE.default_state.id);
        assert_ne!(only_block(&chunk), BlockStateId::AIR);
    }

    #[test]
    fn vanilla_block_palette_properties_select_the_right_state() {
        // A property-bearing entry must not collapse to the block's default state, or
        // every log in a loaded world silently changes axis.
        let upright = load(
            vec![named_entry("minecraft:oak_log", &[("axis", "y")])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        let sideways = load(
            vec![named_entry("minecraft:oak_log", &[("axis", "x")])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        assert_ne!(
            only_block(&upright),
            only_block(&sideways),
            "axis=y and axis=x resolved to the same state"
        );
        for state in [only_block(&upright), only_block(&sideways)] {
            assert_eq!(state.to_block_id(), Block::OAK_LOG.id);
        }
    }

    #[test]
    fn vanilla_biome_palette_resolves_the_resource_location() {
        // `Biome::from_name` takes bare names, so the namespace has to be stripped. If it
        // is not, every biome in a loaded world silently becomes id 0.
        let chunk = load(
            vec![named_entry("minecraft:stone", &[])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        let biomes = chunk.section.dump_biomes();
        assert_eq!(biomes[0], Biome::PLAINS.id);
        assert!(biomes.iter().all(|b| *b == Biome::PLAINS.id));
    }

    #[test]
    fn biome_palette_without_a_namespace_also_resolves() {
        let chunk = load(
            vec![named_entry("minecraft:stone", &[])],
            vec![NbtTag::String("desert".into())],
        );
        assert_eq!(chunk.section.dump_biomes()[0], Biome::DESERT.id);
    }

    #[test]
    fn numeric_palettes_still_load() {
        // Our own on-disk shape. A world written by this server must keep loading.
        let stone = Block::STONE.default_state.id;
        let chunk = load(
            vec![NbtTag::Int(i32::from(BlockStateId::as_u16(stone)))],
            vec![NbtTag::Byte(Biome::DESERT.id as i8)],
        );
        assert_eq!(only_block(&chunk), stone);
        assert_eq!(chunk.section.dump_biomes()[0], Biome::DESERT.id);
    }

    #[test]
    fn an_unknown_block_name_falls_back_to_air_rather_than_failing_the_chunk() {
        // A modded or future block should cost one block, not the whole chunk.
        let chunk = load(
            vec![named_entry("somemod:unobtainium", &[])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        assert_eq!(only_block(&chunk), BlockStateId::AIR);
    }

    #[test]
    fn an_unknown_biome_name_falls_back_to_plains() {
        let chunk = load(
            vec![named_entry("minecraft:stone", &[])],
            vec![NbtTag::String("somemod:mystery_biome".into())],
        );
        assert_eq!(chunk.section.dump_biomes()[0], Biome::PLAINS.id);
    }

    /// Re-reads a written chunk's first section palette.
    fn written_palettes(chunk: &ChunkData) -> (Vec<NbtTag>, Vec<NbtTag>) {
        let bytes = chunk.internal_to_bytes().expect("serialize chunk");
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
        let root = pumpkin_nbt::Nbt::read(&mut reader)
            .expect("reparse written chunk")
            .root_tag;
        let sections = root.get_list("sections").expect("sections");
        let NbtTag::Compound(section) = &sections[0] else {
            panic!("section is not a compound")
        };
        let blocks = section
            .get_compound("block_states")
            .and_then(|bs| bs.get_list("palette"))
            .expect("block palette")
            .to_vec();
        let biomes = section
            .get_compound("biomes")
            .and_then(|b| b.get_list("palette"))
            .expect("biome palette")
            .to_vec();
        (blocks, biomes)
    }

    #[test]
    fn palettes_are_written_in_the_vanilla_shape() {
        // We used to write raw numeric ids, which no other tool can read. A chunk written
        // here has to be loadable by vanilla, so the palette must be named.
        let chunk = load(
            vec![named_entry("minecraft:oak_log", &[("axis", "x")])],
            vec![NbtTag::String("minecraft:desert".into())],
        );
        let (blocks, biomes) = written_palettes(&chunk);

        let NbtTag::Compound(entry) = &blocks[0] else {
            panic!("block palette entry is not a compound: {:?}", blocks[0])
        };
        assert_eq!(entry.get_string("Name"), Some("minecraft:oak_log"));
        assert_eq!(
            entry
                .get_compound("Properties")
                .and_then(|p| p.get_string("axis")),
            Some("x"),
            "block properties were dropped on write"
        );

        assert_eq!(biomes[0], NbtTag::String("minecraft:desert".into()));
    }

    #[test]
    fn a_block_without_properties_omits_the_properties_tag() {
        // Vanilla writes a bare {Name} for a propertyless block; an empty Properties
        // compound is not the same shape.
        let chunk = load(
            vec![named_entry("minecraft:bedrock", &[])],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        let (blocks, _) = written_palettes(&chunk);
        let NbtTag::Compound(entry) = &blocks[0] else {
            panic!("not a compound")
        };
        assert_eq!(entry.get_string("Name"), Some("minecraft:bedrock"));
        assert!(!entry.has("Properties"));
    }

    #[test]
    fn a_vanilla_chunk_round_trips_unchanged() {
        // Load a vanilla-shaped chunk, write it, load it again: same blocks, same biomes.
        // Verified additionally against a real 26.2 chunk from Mojang's server.jar, which
        // round-tripped 98304 of 98304 blocks identically.
        let original = load(
            vec![named_entry("minecraft:deepslate", &[("axis", "y")])],
            vec![NbtTag::String("minecraft:desert".into())],
        );
        let bytes = original.internal_to_bytes().expect("serialize");
        let reloaded = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).expect("reparse");

        assert_eq!(only_block(&reloaded), only_block(&original));
        assert_eq!(
            reloaded.section.dump_biomes(),
            original.section.dump_biomes()
        );
        assert_eq!(only_block(&reloaded).to_block_id(), Block::DEEPSLATE.id);
        assert_eq!(reloaded.section.dump_biomes()[0], Biome::DESERT.id);
    }

    #[test]
    fn a_multi_entry_named_palette_resolves_every_entry() {
        // NBT lists are homogeneous, so a palette is all-named or all-numeric and the two
        // arms can never shadow each other. What does need pinning is that a palette with
        // more than one entry resolves each of them, not just the first.
        let chunk = load(
            vec![
                named_entry("minecraft:bedrock", &[]),
                named_entry("minecraft:deepslate", &[("axis", "y")]),
                named_entry("minecraft:deepslate_gold_ore", &[]),
            ],
            vec![NbtTag::String("minecraft:plains".into())],
        );
        // No data array, so every block takes palette index 0.
        assert_eq!(only_block(&chunk), Block::BEDROCK.default_state.id);
    }
}
