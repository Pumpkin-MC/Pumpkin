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
use pumpkin_data::{Block, BlockStateId, chunk::ChunkStatus, fluid::Fluid};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

use crate::{
    chunk::{
        ChunkEntityData, ChunkReadingError, ChunkSerializingError,
        format::anvil::{SingleChunkDataSerializer, WORLD_DATA_VERSION},
        io::{Dirtiable, file_manager::PathFromLevelFolder},
    },
    generation::section_coords,
    level::LevelFolder,
    tick::{ScheduledTick, TickPriority, scheduler::ChunkTickScheduler},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;

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
        Box::pin(async move { Ok(self.internal_to_bytes()) })
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
                .map(|t| {
                    let val = match t {
                        pumpkin_nbt::tag::NbtTag::Int(x) => *x as u16,
                        pumpkin_nbt::tag::NbtTag::Short(x) => *x as u16,
                        pumpkin_nbt::tag::NbtTag::Byte(x) => *x as u16,
                        pumpkin_nbt::tag::NbtTag::Long(x) => *x as u16,
                        _ => 0,
                    };
                    BlockStateId::new_or_air(val)
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
    let delay = i64::from(nbt.get_int("t")?);
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
        let mut unknown_nbt = root_tag.clone();
        for key in [
            "DataVersion",
            "xPos",
            "zPos",
            "yPos",
            "Status",
            "Heightmaps",
            "sections",
            "block_ticks",
            "fluid_ticks",
            "block_entities",
            "isLightOn",
            "InhabitedTime",
        ] {
            unknown_nbt.child_tags.remove(key);
        }

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
        let mut sky_light_present = vec![false; section_count];
        let mut block_palettes = vec![BlockPalette::default(); section_count];
        let mut biome_palettes = vec![BiomePalette::default(); section_count];
        let mut unknown_section_nbt = vec![NbtCompound::new(); section_count];

        if let Some(sections_list) = root_tag.get_list("sections") {
            for section_tag in sections_list {
                if let pumpkin_nbt::tag::NbtTag::Compound(section_compound) = section_tag {
                    let y = section_compound.get_byte("Y").unwrap_or(0);
                    let index = (y as i32 - min_y_section) as usize;
                    if index >= section_count {
                        continue;
                    }

                    let mut unknown = section_compound.clone();
                    for key in ["Y", "BlockLight", "SkyLight"] {
                        unknown.child_tags.remove(key);
                    }
                    for key in ["block_states", "biomes"] {
                        match unknown.child_tags.get_mut(key) {
                            Some(pumpkin_nbt::tag::NbtTag::Compound(compound)) => {
                                compound.child_tags.remove("data");
                                compound.child_tags.remove("palette");
                            }
                            // Pumpkin serializes these known fields as compounds, so discard an
                            // incompatible representation rather than preventing the rewrite.
                            Some(_) => {
                                unknown.child_tags.remove(key);
                            }
                            None => {}
                        }
                    }
                    unknown_section_nbt[index] = unknown;

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
                    if let Some(sky_light) = sky_light {
                        sky_lights[index] = LightContainer::Full(sky_light);
                        sky_light_present[index] = true;
                    }

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

        // Vanilla only writes a `SkyLight` array for a section it actually touched
        // while lighting; an omitted array is not a stored "0", it is derived from
        // position. Sections above the highest section that carries a `SkyLight` tag
        // see open sky and are 15. Sections at or below it that lack a tag repeat the
        // nearest tagged layer above them, since the light engine skips writing a
        // section whose value is identical to the one above it. If no section in the
        // chunk carries a `SkyLight` tag at all, this dimension has no sky light and
        // every section is dark (0).
        if let Some(top) = sky_light_present.iter().rposition(|&present| present) {
            for light in sky_lights.iter_mut().skip(top + 1) {
                *light = LightContainer::new_empty(15);
            }
            let mut nearest_above = sky_lights[top].clone();
            for i in (0..top).rev() {
                if sky_light_present[i] {
                    nearest_above = sky_lights[i].clone();
                } else {
                    sky_lights[i] = nearest_above.clone();
                }
            }
        } else {
            for light in &mut sky_lights {
                *light = LightContainer::new_empty(0);
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
            block_sections: RwLock::new(block_palettes.into_boxed_slice()),
            random_tick_sections: RwLock::new(random_tick_sections),
            randomly_ticking_mask: std::sync::atomic::AtomicU32::new(randomly_ticking_mask),
            biome_sections: RwLock::new(biome_palettes.into_boxed_slice()),
            unknown_nbt: RwLock::new(unknown_section_nbt.into_boxed_slice()),
            min_y,
        };

        let heightmaps = root_tag.get_compound("Heightmaps").map_or(
            ChunkHeightmaps {
                world_surface: None,
                motion_blocking: None,
                motion_blocking_no_leaves: None,
                ocean_floor: None,
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
                // Any of these can be absent: vanilla only serializes the
                // heightmap types listed for the chunk's status. Absence is
                // resolved by `prime_missing_heightmaps` below, before anything
                // can mistake it for an empty column.
                ocean_floor: h_compound
                    .get_long_array("OCEAN_FLOOR")
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

        let chunk = Self {
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
            unknown_nbt,
            inhabited_time: AtomicU64::new(root_tag.get_long("InhabitedTime").unwrap_or(0) as u64),
        };

        chunk.prime_missing_heightmaps();

        Ok(chunk)
    }

    #[allow(clippy::too_many_lines)]
    fn internal_to_bytes(&self) -> Bytes {
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
        let unknown_sections_lock = self.section.unknown_nbt.read().unwrap();

        let min_section_y = (self.section.min_y >> 4) as i8;

        let mut root_compound = self.unknown_nbt.clone();
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
        if let Some(ref arr) = heightmap_lock.ocean_floor {
            heightmaps_compound.put("OCEAN_FLOOR", NbtTag::LongArray(arr.to_vec()));
        }
        root_compound.put_compound("Heightmaps", heightmaps_compound);

        let mut sections_list = Vec::new();
        for i in 0..self.section.section_count() {
            let mut section_comp = unknown_sections_lock.get(i).cloned().unwrap_or_default();
            let y_val = i as i8 + min_section_y;
            section_comp.put_byte("Y", y_val);

            // block_states
            let block_states_nbt = block_lock[i].to_disk_nbt();
            let mut bs_comp = match section_comp.child_tags.remove("block_states") {
                Some(NbtTag::Compound(compound)) => compound,
                _ => NbtCompound::new(),
            };
            if let Some(ref data_arr) = block_states_nbt.data {
                bs_comp.put("data", NbtTag::LongArray(data_arr.to_vec()));
            }
            let palette_tags: Vec<NbtTag> = block_states_nbt
                .palette
                .iter()
                .map(|id| NbtTag::Int(BlockStateId::as_u16(*id) as i32))
                .collect();
            bs_comp.put_list("palette", palette_tags);
            section_comp.put_compound("block_states", bs_comp);

            // biomes
            let biomes_nbt = biome_lock[i].to_disk_nbt();
            let mut b_comp = match section_comp.child_tags.remove("biomes") {
                Some(NbtTag::Compound(compound)) => compound,
                _ => NbtCompound::new(),
            };
            if let Some(ref data_arr) = biomes_nbt.data {
                b_comp.put("data", NbtTag::LongArray(data_arr.to_vec()));
            }
            let biome_palette_tags: Vec<NbtTag> = biomes_nbt
                .palette
                .iter()
                .map(|&val| NbtTag::Byte(val as i8))
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
            tick_comp.put_int(
                "t",
                i32::try_from(tick.delay).expect("scheduled tick delay must fit vanilla's NBT int"),
            );
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
            tick_comp.put_int(
                "t",
                i32::try_from(tick.delay).expect("scheduled tick delay must fit vanilla's NBT int"),
            );
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
        root_compound.put_long(
            "InhabitedTime",
            self.inhabited_time.load(Ordering::Relaxed) as i64,
        );

        let nbt = pumpkin_nbt::Nbt::from(root_compound);
        nbt.write()
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
        let mut cursor = std::io::Cursor::new(chunk_data);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = if is_named {
            pumpkin_nbt::Nbt::read(&mut reader)
        } else {
            pumpkin_nbt::Nbt::read_unnamed(&mut reader)
        }
        .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        let pos_array = match (nbt.get_int("Position-X"), nbt.get_int("Position-Z")) {
            (Some(x), Some(z)) => [x, z],
            _ => {
                if let Some(pumpkin_nbt::tag::NbtTag::IntArray(pos)) = nbt.get("Position") {
                    if pos.len() >= 2 {
                        [pos[0], pos[1]]
                    } else {
                        [0, 0]
                    }
                } else {
                    [0, 0]
                }
            }
        };

        if pos_array[0] != position.x || pos_array[1] != position.y {
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for entity chunk {},{} but got it for {},{}!",
                position.x, position.y, pos_array[0], pos_array[1],
            )));
        }

        let entities = match nbt.get("Entities") {
            Some(pumpkin_nbt::tag::NbtTag::List(list)) => list
                .iter()
                .filter_map(|t| match t {
                    pumpkin_nbt::tag::NbtTag::Compound(c) => Some(c.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        Ok(Self {
            x: position.x,
            z: position.y,
            data: Mutex::new(entities),
            dirty: AtomicBool::new(false),
        })
    }

    async fn internal_to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        let mut root = NbtCompound::new();
        root.put_int("DataVersion", WORLD_DATA_VERSION);
        root.put(
            "Position",
            pumpkin_nbt::tag::NbtTag::IntArray(vec![self.x, self.z]),
        );
        let entities_tag: Vec<pumpkin_nbt::tag::NbtTag> = self
            .data
            .lock()
            .await
            .iter()
            .map(|c| pumpkin_nbt::tag::NbtTag::Compound(c.clone()))
            .collect();
        root.put_list("Entities", entities_tag);

        let nbt = pumpkin_nbt::Nbt::from(root);
        Ok(nbt.write())
    }
}

#[derive(Clone)]
pub struct ChunkSectionBiomes {
    pub(crate) data: Option<Box<[i64]>>,
    pub(crate) palette: Box<[u8]>,
}

#[derive(Clone)]
pub struct ChunkSectionBlockStates {
    pub(crate) data: Option<Box<[i64]>>,
    pub(crate) palette: Box<[BlockStateId]>,
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

#[cfg(test)]
pub(crate) mod chunk_codec_tests {
    use super::*;
    use crate::chunk::ChunkHeightmapType;
    use pumpkin_nbt::tag::NbtTag;

    fn full_sky_light(value: u8) -> Box<[i8]> {
        let byte = ((value << 4) | value) as i8;
        vec![byte; LightContainer::ARRAY_SIZE].into_boxed_slice()
    }

    fn section(y: i8, sky_light: Option<u8>) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_byte("Y", y);
        if let Some(value) = sky_light {
            compound.put("SkyLight", NbtTag::ByteArray(full_sky_light(value)));
        }
        NbtTag::Compound(compound)
    }

    /// Encodes an overworld-shaped chunk whose lowest 8 sections are solid
    /// stone and which carries a `Heightmaps` compound *without* a
    /// `WORLD_SURFACE` entry - the shape vanilla writes for any chunk saved
    /// below `minecraft:full` that already contains terrain.
    pub fn encode_terrain_chunk_without_world_surface(chunk_x: i32, chunk_z: i32) -> Vec<u8> {
        const MIN_SECTION: i8 = -4;
        const MAX_SECTION: i8 = 19;
        const TOP_STONE_SECTION: i8 = 3;

        let stone = i32::from(Block::STONE.default_state.id.as_u16());

        let mut sections = Vec::new();
        for y in MIN_SECTION..=MAX_SECTION {
            let mut compound = NbtCompound::new();
            compound.put_byte("Y", y);
            if y <= TOP_STONE_SECTION {
                let mut block_states = NbtCompound::new();
                // Single-entry palette and no `data`: a uniform section.
                block_states.put("palette", NbtTag::IntArray(vec![stone]));
                compound.put_compound("block_states", block_states);
            }
            sections.push(NbtTag::Compound(compound));
        }

        let mut root = NbtCompound::new();
        root.put_int("xPos", chunk_x);
        root.put_int("zPos", chunk_z);
        root.put_int("yPos", i32::from(MIN_SECTION));
        root.put_list("sections", sections);
        root.put_string("Status", "minecraft:carvers".to_string());
        // Present but empty, exactly as a pre-`full` vanilla chunk with no
        // status-eligible heightmap types is written.
        root.put_compound("Heightmaps", NbtCompound::new());

        pumpkin_nbt::Nbt::from(root).write().to_vec()
    }

    #[test]
    fn missing_world_surface_heightmap_is_recomputed_from_blocks() {
        let bytes = encode_terrain_chunk_without_world_surface(0, 0);
        let chunk = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).unwrap();
        let min_y = chunk.section.min_y;
        let heightmap = chunk.heightmap.lock().unwrap();

        // Stone fills sections -4..=3, so the topmost non-air block is y = 63.
        // Without priming, `get` answers `min_y - 1` (-65), which reads as "this
        // column has no terrain at all" and makes the sky light producer flood
        // the entire column with 15.
        for (x, z) in [(0, 0), (7, 9), (15, 15)] {
            assert_eq!(
                heightmap.get(ChunkHeightmapType::WorldSurface, x, z, min_y),
                63,
                "column ({x}, {z}) must report its real surface, not the empty-column sentinel"
            );
        }
    }

    fn encode_chunk(min_y_section: i32, sections: Vec<NbtTag>) -> Vec<u8> {
        let mut root = NbtCompound::new();
        root.put_int("xPos", 0);
        root.put_int("zPos", 0);
        root.put_int("yPos", min_y_section);
        root.put_list("sections", sections);
        root.put_bool("isLightOn", true);

        pumpkin_nbt::Nbt::from(root).write().to_vec()
    }

    #[test]
    fn missing_sky_light_above_terrain_derives_open_sky() {
        // Only the middle section carries a `SkyLight` tag, as a vanilla chunk
        // would for terrain with air above it.
        let sections = vec![
            section(0, None),
            section(1, Some(7)),
            section(2, None),
            section(3, None),
        ];
        let bytes = encode_chunk(0, sections);
        let chunk = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).unwrap();
        let light = chunk.light_engine.lock().unwrap();

        // Sections above the highest tagged section see open sky (15), not the
        // buggy default of reading a missing tag as `Empty(0)`.
        assert_eq!(light.sky_light[2].get(0, 0, 0), 15);
        assert_eq!(light.sky_light[3].get(0, 0, 0), 15);

        // A section below the highest tagged one with no tag of its own repeats
        // the nearest tagged layer above it, not a fixed default.
        assert_eq!(light.sky_light[0].get(0, 0, 0), 7);

        // The tagged section itself round-trips unchanged.
        assert_eq!(light.sky_light[1].get(0, 0, 0), 7);
    }

    #[test]
    fn no_sky_light_tags_reads_as_dark_dimension() {
        // No section carries a `SkyLight` tag at all, as in a dimension without
        // sky light (e.g. the Nether). This must not be lit up as open sky.
        let sections = vec![section(0, None), section(1, None)];
        let bytes = encode_chunk(0, sections);
        let chunk = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).unwrap();
        let light = chunk.light_engine.lock().unwrap();

        assert_eq!(light.sky_light[0].get(0, 0, 0), 0);
        assert_eq!(light.sky_light[1].get(0, 0, 0), 0);
    }

    #[test]
    fn preserves_unknown_root_tags_when_reserializing() {
        let mut root = NbtCompound::new();
        root.put_int("xPos", 0);
        root.put_int("zPos", 0);
        root.put_int("yPos", 0);
        root.put_list("sections", vec![section(0, None)]);

        let mut future_data = NbtCompound::new();
        future_data.put_string("owner", "vanilla".to_string());
        root.put_compound("FutureData", future_data.clone());

        let bytes = pumpkin_nbt::Nbt::from(root).write();
        let chunk = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).unwrap();
        let encoded = chunk.internal_to_bytes();
        let mut cursor = std::io::Cursor::new(encoded.as_ref());
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
        let decoded = pumpkin_nbt::Nbt::read(&mut reader).unwrap();

        assert_eq!(
            decoded.root_tag.get_compound("FutureData"),
            Some(&future_data)
        );
        assert_eq!(
            decoded.root_tag.get_int("DataVersion"),
            Some(WORLD_DATA_VERSION)
        );
    }

    #[test]
    fn preserves_unknown_section_tags_when_reserializing() {
        let mut section_compound = NbtCompound::new();
        section_compound.put_byte("Y", 0);
        section_compound.put_string("FutureSectionField", "retained".to_string());

        let mut block_states = NbtCompound::new();
        block_states.put_list("palette", vec![NbtTag::Int(0)]);
        block_states.put_int("FutureBlockStatesField", 42);
        section_compound.put_compound("block_states", block_states);

        let mut biomes = NbtCompound::new();
        biomes.put_list("palette", vec![NbtTag::Byte(0)]);
        biomes.put_string("FutureBiomesField", "retained".to_string());
        section_compound.put_compound("biomes", biomes);

        let bytes = encode_chunk(0, vec![NbtTag::Compound(section_compound)]);
        let chunk = ChunkData::internal_from_bytes(&bytes, Vector2::new(0, 0)).unwrap();
        let encoded = chunk.internal_to_bytes();
        let mut cursor = std::io::Cursor::new(encoded.as_ref());
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
        let decoded = pumpkin_nbt::Nbt::read(&mut reader).unwrap();
        let sections = decoded.root_tag.get_list("sections").unwrap();
        let NbtTag::Compound(section) = &sections[0] else {
            panic!("serialized section must be a compound");
        };

        assert_eq!(section.get_string("FutureSectionField"), Some("retained"));
        assert_eq!(
            section
                .get_compound("block_states")
                .and_then(|block_states| block_states.get_int("FutureBlockStatesField")),
            Some(42)
        );
        assert_eq!(
            section
                .get_compound("biomes")
                .and_then(|biomes| biomes.get_string("FutureBiomesField")),
            Some("retained")
        );
    }
}
