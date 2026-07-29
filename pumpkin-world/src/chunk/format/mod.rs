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
use pumpkin_nbt::{compound::NbtCompound, nbt_long_array};
use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

use crate::{
    chunk::{
        ChunkEntityData, ChunkReadingError, ChunkSerializingError,
        format::anvil::{SingleChunkDataSerializer, WORLD_DATA_VERSION},
        io::{Dirtiable, file_manager::PathFromLevelFolder},
    },
    chunk_system::chunk_state::StagedChunkEnum,
    generation::section_coords,
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

/// Root NBT keys that `ChunkData::internal_to_bytes` writes itself.
///
/// Some are recomputed from live state (`DataVersion`, `xPos`/`yPos`/`zPos`, `Status`,
/// `sections`, `isLightOn`, `InhabitedTime`); the rest are echoed back out of a typed
/// field that `internal_from_bytes` parsed them into (`Heightmaps`, `block_ticks`,
/// `fluid_ticks`, `block_entities`). Either way the writer owns them, so they must
/// never be sourced from [`ChunkData::retained_nbt`].
///
/// Keep in sync with the `root_compound.put_*` calls in `internal_to_bytes`;
/// `writer_emits_exactly_the_written_root_keys` fails on drift.
const WRITTEN_ROOT_KEYS: &[&str] = &[
    "DataVersion",
    "xPos",
    "yPos",
    "zPos",
    "Status",
    "Heightmaps",
    "sections",
    "block_ticks",
    "fluid_ticks",
    "block_entities",
    "isLightOn",
    "InhabitedTime",
];

/// Root keys we neither write nor carry over, because writing them back would be worse
/// than losing them.
///
/// The rule: retain keys that *describe* what is already in the chunk, drop keys that
/// record *work still to be done*. We have no code for any key below, so we can never
/// clear one once it has been applied — we would re-assert the same directive on every
/// save, against blocks that have since changed.
///
/// - `entities`: entities live in the separate `entities/` region folder, so a retained
///   copy would be a second, never-updated set of the same mobs.
/// - `PostProcessing`, `below_zero_retrogen`, `UpgradeData`: work records.
/// - `carving_mask`: generation-stage scratch state.
/// - `Lights`: vanilla's per-section list of light sources still to be processed,
///   written for chunks below `light` status. Retaining it would put a work record on a
///   chunk we go on to stamp `Status: minecraft:full`.
///
/// `carving_mask` and `Lights` only became reachable when the retention cutoff moved from
/// `full` down to `features`; both belong to chunks caught mid-generation.
///
/// How these are classified is reasoning about vanilla rather than something validated
/// against a real region file. What is verified is that we have no code for any of them.
const NEVER_RETAINED_ROOT_KEYS: &[&str] = &[
    "entities",
    "PostProcessing",
    "below_zero_retrogen",
    "UpgradeData",
    "carving_mask",
    "Lights",
];

fn is_owned_root_key(key: &str) -> bool {
    WRITTEN_ROOT_KEYS.contains(&key) || NEVER_RETAINED_ROOT_KEYS.contains(&key)
}

/// Clones the root tags we do not model out of a freshly parsed chunk, so they can be
/// written back untouched instead of dropped.
fn retain_foreign_root_tags(root: &NbtCompound) -> NbtCompound {
    root.child_tags
        .iter()
        .filter(|&(key, _)| !is_owned_root_key(key))
        .map(|(key, tag)| (key.clone(), tag.clone()))
        .collect()
}

/// Writes retained tags back onto a root compound.
///
/// Must be called *after* every `root_compound.put_*` in `internal_to_bytes`:
/// [`NbtCompound::put`] is insert-if-absent, so merging last makes it structurally
/// impossible for a stale disk value to shadow live chunk state. A bug in the filter can
/// then only ever drop a foreign tag, never corrupt an owned one.
///
/// Deliberately not `Extend`: `NbtCompound` has two `Extend` impls that disagree about
/// overwriting, and the call site cannot show which one is selected.
fn merge_foreign_root_tags(root: &mut NbtCompound, retained: &NbtCompound) {
    for (key, tag) in &retained.child_tags {
        root.put(key, tag.clone());
    }
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

        // Retain only from a chunk that will be written back over the same terrain the
        // metadata describes.
        //
        // A chunk below `features` goes back into the generation pipeline through
        // `ProtoChunk::from_chunk_data`, which restores blocks and heightmaps but leaves
        // `structure_starts` empty and `carving_mask` fresh, so it is finished *without*
        // the structures the retained metadata claims are there. An unrecognised `Status`
        // falls through to `Empty` above and lands here too — precisely the case where we
        // must not vouch for anything.
        //
        // At or above `features` that cannot happen: the features/structures step never
        // re-runs for such a chunk. `drop_satisfied_tasks` deletes every task at or below
        // the loaded stage, `Cache::advance` early-returns for a stage already reached,
        // and the dispatch loop drops queued nodes for reached stages. `structure_starts`
        // is only ever filled by `set_structure_starts`/`set_structure_references`, whose
        // tasks are dropped for anything loaded at or above `StructureReferences`.
        //
        // So `features` is the boundary, not `full` — gating on `full` made this a no-op
        // for exactly the chunks #2626 reported, which sit at `initialize_light`.
        let retained_nbt = if StagedChunkEnum::from(status) >= StagedChunkEnum::Features {
            retain_foreign_root_tags(&root_tag)
        } else {
            NbtCompound::new()
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
            retained_nbt,
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
                .map(|id| NbtTag::Int(BlockStateId::as_u16(*id) as i32))
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

        // The live value, never a retained one — this is incremented every tick in
        // `World::tick`. Before this it was parsed on load and then never written, so the
        // first save zeroed whatever had accumulated.
        root_compound.put_long(
            "InhabitedTime",
            self.inhabited_time.load(Ordering::Relaxed) as i64,
        );

        // Last. See `merge_foreign_root_tags` — `put` is insert-if-absent, so an owned key
        // can never be shadowed by a stale value off disk.
        merge_foreign_root_tags(&mut root_compound, &self.retained_nbt);

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
    use super::{NEVER_RETAINED_ROOT_KEYS, WORLD_DATA_VERSION, WRITTEN_ROOT_KEYS};
    use crate::chunk::ChunkData;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_nbt::tag::NbtTag;
    use pumpkin_util::math::vector2::Vector2;
    use std::sync::atomic::Ordering;

    /// The smallest root compound `internal_from_bytes` accepts. An empty `sections` list
    /// is valid: `max_y_section` starts at `min_y_section`, so the section count is 1.
    fn minimal_root(status: &str) -> NbtCompound {
        let mut root = NbtCompound::new();
        root.put_int("DataVersion", WORLD_DATA_VERSION);
        root.put_int("xPos", 0);
        root.put_int("zPos", 0);
        root.put_int("yPos", -4);
        root.put_string("Status", status.to_string());
        root.put_list("sections", Vec::new());
        root
    }

    fn encode(root: &NbtCompound) -> Vec<u8> {
        let mut buf = Vec::new();
        pumpkin_nbt::serializer::to_bytes(root, &mut buf).expect("serialize fixture");
        buf
    }

    fn decode_root(bytes: &[u8]) -> NbtCompound {
        let mut cursor = std::io::Cursor::new(bytes);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
        pumpkin_nbt::Nbt::read(&mut reader)
            .expect("reparse written chunk")
            .root_tag
    }

    fn parse(root: &NbtCompound) -> ChunkData {
        ChunkData::internal_from_bytes(&encode(root), Vector2::new(0, 0)).expect("parse fixture")
    }

    fn round_trip(root: &NbtCompound) -> NbtCompound {
        decode_root(&parse(root).internal_to_bytes().expect("serialize chunk"))
    }

    fn sorted_keys(root: &NbtCompound) -> Vec<&str> {
        let mut keys: Vec<&str> = root.child_tags.keys().map(AsRef::as_ref).collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn foreign_root_tags_survive_a_disk_round_trip() {
        let mut fixture = minimal_root("minecraft:full");

        let mut structures = NbtCompound::new();
        structures.put_compound("starts", NbtCompound::new());
        structures.put_compound("References", NbtCompound::new());
        fixture.put_compound("structures", structures);
        fixture.put("LastUpdate", NbtTag::Long(1_287_805));
        fixture.put_compound("blending_data", NbtCompound::new());
        // A tag no version of this server knows about, to pin the future-proofing claim:
        // an unrecognised key survives with no code change. The list of empty lists is the
        // load-bearing shape here, being the only nested empty sequence.
        fixture.put_list(
            "some_future_mojang_tag",
            (0..24).map(|_| NbtTag::List(Vec::new())).collect(),
        );

        let out = round_trip(&fixture);

        for key in [
            "structures",
            "LastUpdate",
            "blending_data",
            "some_future_mojang_tag",
        ] {
            assert_eq!(
                out.get(key),
                fixture.get(key),
                "foreign root tag `{key}` was not preserved across a save"
            );
        }
    }

    #[test]
    fn work_directive_root_tags_are_never_retained() {
        let mut fixture = minimal_root("minecraft:full");
        // Spelled out rather than iterated from NEVER_RETAINED_ROOT_KEYS, so that deleting
        // a key from that constant also has to fail here.
        fixture.put_list("entities", vec![NbtTag::Compound(NbtCompound::new())]);
        fixture.put_list("PostProcessing", Vec::new());
        fixture.put_compound("below_zero_retrogen", NbtCompound::new());
        fixture.put_compound("UpgradeData", NbtCompound::new());
        fixture.put("carving_mask", NbtTag::LongArray(vec![1, 2, 3]));
        fixture.put_list(
            "Lights",
            (0..24).map(|_| NbtTag::List(Vec::new())).collect(),
        );

        let out = round_trip(&fixture);

        // `entities` is excluded because entities live in the separate `entities/` region
        // folder; a retained copy would be a second, never-updated set of the same mobs.
        // The rest record work still to be done, which we have no way to perform or clear.
        for key in [
            "entities",
            "PostProcessing",
            "below_zero_retrogen",
            "UpgradeData",
            "carving_mask",
            "Lights",
        ] {
            assert!(!out.has(key), "`{key}` should never be written back");
        }
    }

    #[test]
    fn work_directive_root_tags_are_never_retained_below_full_either() {
        // The exclusions have to hold at the bottom of the retention range too, which is
        // where a half-generated vanilla chunk actually carries them. `carving_mask` and
        // `Lights` are unreachable at `full` and reachable here.
        let mut fixture = minimal_root("minecraft:initialize_light");
        fixture.put("carving_mask", NbtTag::LongArray(vec![1, 2, 3]));
        fixture.put_list(
            "Lights",
            (0..24).map(|_| NbtTag::List(Vec::new())).collect(),
        );
        fixture.put_list("PostProcessing", Vec::new());

        let out = round_trip(&fixture);

        for key in ["carving_mask", "Lights", "PostProcessing"] {
            assert!(!out.has(key), "`{key}` should never be written back");
        }
    }

    #[test]
    fn owned_root_keys_are_rewritten_from_the_live_chunk() {
        let mut fixture = minimal_root("minecraft:full");
        fixture.put_int("DataVersion", 1234);

        let out = round_trip(&fixture);

        // Regression test for the insert-if-absent `put`: if `merge_foreign_root_tags` ever
        // moves above the owned puts, `DataVersion` comes back as 1234 and this fails.
        assert_eq!(out.get_int("DataVersion"), Some(WORLD_DATA_VERSION));
        assert_eq!(out.get_string("Status"), Some("minecraft:full"));
        assert_eq!(out.get_int("xPos"), Some(0));
        assert_eq!(out.get_int("zPos"), Some(0));
        assert_eq!(out.get_int("yPos"), Some(-4));
    }

    #[test]
    fn heightmaps_round_trip_verbatim_through_the_typed_field() {
        let mut heightmaps = NbtCompound::new();
        heightmaps.put("WORLD_SURFACE", NbtTag::LongArray(vec![7; 37]));
        let mut fixture = minimal_root("minecraft:full");
        fixture.put_compound("Heightmaps", heightmaps);

        let out = round_trip(&fixture);

        // Heightmaps are parsed into `ChunkHeightmaps` and re-emitted from it; nothing
        // recomputes them on the load path. So they are owned by the writer but not
        // *derived* by it, and they never route through `retained_nbt`.
        let written = out.get_compound("Heightmaps").expect("Heightmaps written");
        assert_eq!(
            written.get_long_array("WORLD_SURFACE"),
            Some(&[7i64; 37][..])
        );
    }

    #[test]
    fn inhabited_time_round_trips_through_disk() {
        let mut fixture = minimal_root("minecraft:full");
        fixture.put_long("InhabitedTime", 123_456);

        assert_eq!(
            parse(&fixture).inhabited_time.load(Ordering::Relaxed),
            123_456
        );
        // Before this fix the key was parsed and never written, so the first save zeroed
        // whatever had accumulated.
        assert_eq!(
            round_trip(&fixture).get_long("InhabitedTime"),
            Some(123_456)
        );
    }

    #[test]
    fn inhabited_time_is_written_from_the_live_value() {
        let mut fixture = minimal_root("minecraft:full");
        fixture.put_long("InhabitedTime", 100);

        let chunk = parse(&fixture);
        chunk.inhabited_time.store(500, Ordering::Relaxed);
        let out = decode_root(&chunk.internal_to_bytes().expect("serialize chunk"));

        assert_eq!(out.get_long("InhabitedTime"), Some(500));
    }

    #[test]
    fn writer_emits_exactly_the_written_root_keys() {
        let out = round_trip(&minimal_root("minecraft:full"));

        let mut expected: Vec<&str> = WRITTEN_ROOT_KEYS.to_vec();
        expected.sort_unstable();
        assert_eq!(sorted_keys(&out), expected);

        // Cross-check against a literal list. Without this, a change that stops writing a
        // key *and* drops it from the constant would pass the assertion above, and that key
        // would silently start being echoed back off disk instead.
        assert_eq!(
            expected,
            [
                "DataVersion",
                "Heightmaps",
                "InhabitedTime",
                "Status",
                "block_entities",
                "block_ticks",
                "fluid_ticks",
                "isLightOn",
                "sections",
                "xPos",
                "yPos",
                "zPos",
            ]
        );
    }

    #[test]
    fn chunks_below_the_features_stage_retain_nothing() {
        let mut fixture = minimal_root("minecraft:carvers");
        fixture.put_compound("structures", NbtCompound::new());
        fixture.put("LastUpdate", NbtTag::Long(1));

        // A chunk below `features` still has its features/structures step ahead of it, so
        // it is finished without the structures this metadata describes. Losing the tags is
        // what we already did; writing them back beside contradicting terrain would be new
        // damage.
        assert!(parse(&fixture).retained_nbt.is_empty());
        let out = round_trip(&fixture);
        assert!(!out.has("structures"));
        assert!(!out.has("LastUpdate"));
    }

    #[test]
    fn chunks_from_the_features_stage_up_retain_their_metadata() {
        // The statuses a half-generated vanilla world actually contains, including the
        // `initialize_light` of #2626's reported chunk. Gating on `full` meant every one of
        // these was still stripped on save.
        for status in [
            "minecraft:features",
            "minecraft:initialize_light",
            "minecraft:light",
            "minecraft:spawn",
            "minecraft:full",
        ] {
            let mut fixture = minimal_root(status);
            fixture.put_compound("structures", NbtCompound::new());
            fixture.put("LastUpdate", NbtTag::Long(1));

            assert!(
                !parse(&fixture).retained_nbt.is_empty(),
                "`{status}` should retain foreign tags"
            );
            let out = round_trip(&fixture);
            assert!(out.has("structures"), "`{status}` dropped `structures`");
            assert!(out.has("LastUpdate"), "`{status}` dropped `LastUpdate`");
        }
    }

    #[test]
    fn unrecognized_status_retains_nothing() {
        let mut fixture = minimal_root("minecraft:some_future_step");
        fixture.put_compound("structures", NbtCompound::new());

        // An unknown status falls through to `ChunkStatus::Empty`, so the terrain is
        // regenerated from scratch — the worst case for vouching for stale metadata.
        assert!(parse(&fixture).retained_nbt.is_empty());
    }

    #[test]
    fn retained_tags_survive_the_proto_chunk_round_trip() {
        use crate::ProtoChunk;
        use crate::chunk_system::chunk_state::Chunk;
        use crate::generation::get_world_gen;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_data::dimension::Dimension;
        use pumpkin_util::world_seed::Seed;

        // Flat, so there is no noise-router setup cost.
        let world_gen = get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            true,
            Vec::new(),
            String::new(),
        );

        // A chunk we generated ourselves pays nothing for any of this.
        assert!(ProtoChunk::new(0, 0, &world_gen).retained_nbt.is_empty());

        let mut fixture = minimal_root("minecraft:full");
        fixture.put_compound("structures", NbtCompound::new());
        fixture.put_long("InhabitedTime", 777);

        // The level -> proto -> level trip a full chunk makes when it is downgraded for
        // relighting. Without the carry on `ProtoChunk`, both assertions below fail and the
        // fix is silently vacuous on this path.
        let mut chunk = Chunk::Proto(Box::new(ProtoChunk::from_chunk_data(
            &parse(&fixture),
            &world_gen,
        )));
        chunk.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(rebuilt) = chunk else {
            panic!("upgrade_to_level_chunk did not produce a level chunk");
        };

        let out = decode_root(&rebuilt.internal_to_bytes().expect("serialize chunk"));
        assert_eq!(out.get("structures"), fixture.get("structures"));
        assert_eq!(out.get_long("InhabitedTime"), Some(777));
        // Deliberately asserts nothing about section counts: #2551 changes
        // `ProtoChunk::new`'s height source, which changes them.
    }

    #[test]
    fn owned_keys_are_disjoint_from_never_retained_keys() {
        for key in NEVER_RETAINED_ROOT_KEYS {
            assert!(
                !WRITTEN_ROOT_KEYS.contains(key),
                "`{key}` is both written and never-retained; the writer would emit it \
                 while the filter claims we drop it"
            );
        }
    }
}
