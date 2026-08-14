use std::sync::Arc;

use pumpkin_data::block_properties::is_air;
use pumpkin_data::chunk::DoublePerlinNoiseParameters;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::structures::{Structure, StructureKeys, StructureSet};
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::{Block, BlockState, block_properties::blocks_movement, chunk::Biome};
use pumpkin_data::{BlockId, BlockStateId, tag};
use pumpkin_util::{
    HeightMap,
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, get_decorator_seed, xoroshiro128::Xoroshiro},
};
use rustc_hash::FxHashMap;

use super::{
    GlobalRandomConfig, biome_coords,
    noise::router::proto_noise_router::DoublePerlinNoiseBuilder,
    positions::chunk_pos::{start_block_x, start_block_z},
    surface::terrain::SurfaceTerrainBuilder,
};
use crate::chunk::format::LightContainer;
use crate::chunk::{ChunkData, ChunkHeightmapType, ChunkLight};
use crate::chunk_system::StagedChunkEnum;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::noise::CHUNK_DIM;
use crate::generation::noise::aquifer_sampler::{FluidLevel, FluidLevelSamplerImpl};
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::section_coords::section_to_block;
use crate::generation::structure::structures::StructureInstance;
use crate::{
    chunk::CHUNK_AREA,
    generation::{biome, positions::chunk_pos},
    world::{BlockAccessor, WorldPortalExt},
};
use pumpkin_data::tag::get_tag_ids;
use pumpkin_nbt::compound::NbtCompound;

use crate::generation::structure::template::BlockPlacer;
use crate::tick::{ScheduledTick, TickPriority};

pub trait GenerationCache: HeightLimitView + BlockAccessor {
    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk;
    fn get_center_chunk(&self) -> &ProtoChunk;

    fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk>;
    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk>;

    fn try_get_proto_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk>;

    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId;
    fn get_fluid_and_fluid_state(&self, position: &Vector3<i32>) -> (Fluid, FluidState);
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState);
    fn add_block_entity(&mut self, pos: &Vector3<i32>, nbt: NbtCompound);
    fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32;
    fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn is_air(&self, local_pos: &Vector3<i32>) -> bool;
    fn get_biome_for_terrain_gen(&self, x: i32, y: i32, z: i32) -> &'static Biome;
    fn get_blending_data(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&crate::generation::blender::blending_data::BlendingData>;
}

const AIR_BLOCK: Block = Block::AIR;

pub struct StandardChunkFluidLevelSampler {
    top_fluid: FluidLevel,
    bottom_fluid: FluidLevel,
    bottom_y: i32,
}

impl StandardChunkFluidLevelSampler {
    #[must_use]
    pub fn new(top_fluid: FluidLevel, bottom_fluid: FluidLevel) -> Self {
        let bottom_y = top_fluid
            .max_y_exclusive()
            .min(bottom_fluid.max_y_exclusive());
        Self {
            top_fluid,
            bottom_fluid,
            bottom_y,
        }
    }
}

impl FluidLevelSamplerImpl for StandardChunkFluidLevelSampler {
    fn get_fluid_level(&self, _x: i32, y: i32, _z: i32) -> &FluidLevel {
        if y < self.bottom_y {
            &self.bottom_fluid
        } else {
            &self.top_fluid
        }
    }
}

pub struct ProtoChunk {
    pub x: i32,
    pub z: i32,
    pub default_block: &'static BlockState,
    biome_mixer_seed: i64,
    pub(crate) flat_block_map: Box<[BlockStateId]>,
    pub flat_biome_map: Box<[u8]>,
    pub flat_surface_height_map: [i16; CHUNK_AREA],
    pub flat_ocean_floor_height_map: [i16; CHUNK_AREA],
    pub flat_motion_blocking_height_map: [i16; CHUNK_AREA],
    pub flat_motion_blocking_no_leaves_height_map: [i16; CHUNK_AREA],
    pub structure_starts: FxHashMap<StructureKeys, StructureInstance>,

    height: u16,
    bottom_y: i8,
    generation_height: u16,
    generation_bottom_y: i8,
    pub stage: StagedChunkEnum,
    pub light: ChunkLight,
    pub carving_mask: crate::generation::carver::mask::CarvingMask,
    pub blending_data: Option<crate::generation::blender::blending_data::BlendingData>,
    pub pending_block_entities: Vec<NbtCompound>,
    pending_structure_entities: Vec<NbtCompound>,
    pub fluid_ticks: Vec<ScheduledTick<&'static Fluid>>,
}

pub struct TerrainCache {
    pub terrain_builder: SurfaceTerrainBuilder,
    pub surface_noise: DoublePerlinNoiseSampler,
    pub secondary_noise: DoublePerlinNoiseSampler,
}

impl TerrainCache {
    #[must_use]
    pub fn from_random(random_config: &GlobalRandomConfig) -> Self {
        let random = &random_config.base_random_deriver;
        let terrain_builder = SurfaceTerrainBuilder::new(random);
        let surface_noise = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
            &random_config.base_random_deriver,
            &DoublePerlinNoiseParameters::SURFACE,
        );
        let secondary_noise = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
            &random_config.base_random_deriver,
            &DoublePerlinNoiseParameters::SURFACE_SECONDARY,
        );
        Self {
            terrain_builder,
            surface_noise,
            secondary_noise,
        }
    }
}

impl ProtoChunk {
    #[cfg(test)]
    pub(crate) fn has_structure(&self, key: StructureKeys) -> bool {
        self.structure_starts.contains_key(&key)
    }

    #[must_use]
    pub fn new(x: i32, z: i32, generator: &dyn super::generator::ChunkGenerator) -> Self {
        let dimension = generator.dimension();
        let height = dimension.height as u16;
        let bottom_y = dimension.min_y as i8;
        let section_count = (height as usize) / 16;
        let (generation_height, generation_bottom_y) = generator.generation_bounds();
        let default_block = generator.default_block();
        let biome_mixer_seed = generator.biome_mixer_seed();

        let default_heightmap = [i16::MIN; CHUNK_AREA];
        Self {
            x,
            z,
            default_block,
            biome_mixer_seed,
            flat_block_map: vec![BlockStateId::AIR; CHUNK_AREA * height as usize]
                .into_boxed_slice(),
            flat_biome_map: vec![
                Biome::PLAINS.id;
                biome_coords::from_block(CHUNK_DIM as i32) as usize
                    * biome_coords::from_block(CHUNK_DIM as i32) as usize
                    * biome_coords::from_block(height as i32) as usize
            ]
            .into_boxed_slice(),
            flat_surface_height_map: default_heightmap,
            flat_ocean_floor_height_map: default_heightmap,
            flat_motion_blocking_height_map: default_heightmap,
            flat_motion_blocking_no_leaves_height_map: default_heightmap,
            structure_starts: FxHashMap::default(),
            height,
            bottom_y,
            generation_height,
            generation_bottom_y,
            stage: StagedChunkEnum::Empty,
            light: ChunkLight {
                sky_light: (0..section_count)
                    .map(|_| LightContainer::new_empty(0))
                    .collect(),
                block_light: (0..section_count)
                    .map(|_| LightContainer::new_empty(0))
                    .collect(),
            },
            carving_mask: crate::generation::carver::mask::CarvingMask::new(
                height as i32,
                bottom_y as i32,
            ),
            blending_data: None,
            pending_block_entities: Vec::new(),
            pending_structure_entities: Vec::new(),
            fluid_ticks: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_chunk_data(
        chunk_data: &ChunkData,
        generator: &dyn super::generator::ChunkGenerator,
    ) -> Self {
        let mut proto_chunk = Self::new(chunk_data.x, chunk_data.z, generator);

        proto_chunk.light = chunk_data
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        proto_chunk
            .blending_data
            .clone_from(&chunk_data.blending_data);

        let section_data = &chunk_data.section;
        let heightmap_data = chunk_data
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let block_sections_guard = section_data
            .block_sections
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let biome_sections_guard = section_data
            .biome_sections
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        for (section_idx, block_palette) in block_sections_guard.iter().enumerate() {
            let section_base_y = section_idx as i32 * 16;

            if section_base_y >= proto_chunk.height() as i32 {
                continue;
            }

            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        let block_state_id = block_palette.get(x, y, z);
                        let block_state = BlockState::from_id(block_state_id);
                        let absolute_y = section_base_y + y as i32 + section_data.min_y;

                        proto_chunk.set_block_state(x as i32, absolute_y, z as i32, block_state);
                    }
                }
            }

            if let Some(biome_palette) = biome_sections_guard.get(section_idx) {
                for x in 0..4 {
                    for y in 0..4 {
                        for z in 0..4 {
                            let biome_id = biome_palette.get(x, y, z);
                            let biome_y_idx = (section_idx * 4) + y;
                            let index = proto_chunk.local_biome_pos_to_biome_index(
                                x as i32,
                                biome_y_idx as i32,
                                z as i32,
                            );
                            proto_chunk.flat_biome_map[index] = biome_id;
                        }
                    }
                }
            }
        }
        drop(block_sections_guard);
        drop(biome_sections_guard);

        for z in 0..16 {
            for x in 0..16 {
                let index = Self::local_position_to_height_map_index(x, z);

                proto_chunk.flat_motion_blocking_height_map[index] = heightmap_data.get(
                    ChunkHeightmapType::MotionBlocking,
                    x,
                    z,
                    section_data.min_y,
                ) as i16;

                proto_chunk.flat_motion_blocking_no_leaves_height_map[index] = heightmap_data.get(
                    ChunkHeightmapType::MotionBlockingNoLeaves,
                    x,
                    z,
                    section_data.min_y,
                )
                    as i16;

                proto_chunk.flat_surface_height_map[index] =
                    heightmap_data.get(ChunkHeightmapType::WorldSurface, x, z, section_data.min_y)
                        as i16;
            }
        }

        let saved_stage = StagedChunkEnum::from(chunk_data.status);
        proto_chunk.stage = saved_stage;
        if (StagedChunkEnum::StructureStart..StagedChunkEnum::Features).contains(&saved_stage) {
            // Structure starts and references are currently transient proto-chunk data.
            // Rebuild them when resuming a partially generated chunk so structures that
            // cross chunk boundaries are not truncated at the unload boundary.
            proto_chunk.stage = StagedChunkEnum::Biomes;
            generator.rebuild_structure_starts(&mut proto_chunk);
            if saved_stage >= StagedChunkEnum::StructureReferences {
                generator.rebuild_structure_references(&mut proto_chunk);
            }
            proto_chunk.stage = saved_stage;
        }
        proto_chunk
    }

    #[inline]
    #[must_use]
    pub const fn stage_id(&self) -> u8 {
        self.stage as u8
    }

    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    #[must_use]
    pub const fn bottom_y(&self) -> i8 {
        self.bottom_y
    }

    #[must_use]
    pub const fn generation_height(&self) -> u16 {
        self.generation_height
    }

    #[must_use]
    pub const fn generation_bottom_y(&self) -> i8 {
        self.generation_bottom_y
    }

    pub fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.pending_block_entities.push(nbt);
    }

    pub fn take_pending_block_entities(&mut self) -> Vec<NbtCompound> {
        std::mem::take(&mut self.pending_block_entities)
    }

    pub fn add_structure_entity(&mut self, nbt: NbtCompound) {
        self.pending_structure_entities.push(nbt);
    }

    fn take_pending_structure_entities(&mut self) -> Vec<NbtCompound> {
        std::mem::take(&mut self.pending_structure_entities)
    }

    pub fn schedule_fluid_tick(&mut self, x: i32, y: i32, z: i32, fluid: &'static Fluid) {
        self.fluid_ticks.push(ScheduledTick {
            delay: 0,
            priority: TickPriority::Normal,
            position: BlockPos::new(x, y, z),
            value: fluid,
        });
    }

    fn maybe_update_surface_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_surface_height_map[index];
        self.flat_surface_height_map[index] = current_height.max(y);
    }

    fn maybe_update_ocean_floor_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_ocean_floor_height_map[index];
        self.flat_ocean_floor_height_map[index] = current_height.max(y);
    }

    fn maybe_update_motion_blocking_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_motion_blocking_height_map[index];
        self.flat_motion_blocking_height_map[index] = current_height.max(y);
    }

    fn maybe_update_motion_blocking_no_leaves_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_motion_blocking_no_leaves_height_map[index];
        self.flat_motion_blocking_no_leaves_height_map[index] = current_height.max(y);
    }

    #[must_use]
    pub const fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32 {
        match heightmap {
            HeightMap::WorldSurfaceWg | HeightMap::WorldSurface => {
                self.top_block_height_exclusive(x, z)
            }
            HeightMap::OceanFloorWg | HeightMap::OceanFloor => {
                self.ocean_floor_height_exclusive(x, z)
            }
            HeightMap::MotionBlocking => self.top_motion_blocking_block_height_exclusive(x, z),
            HeightMap::MotionBlockingNoLeaves => {
                self.top_motion_blocking_block_no_leaves_height_exclusive(x, z)
            }
        }
    }

    #[must_use]
    pub const fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let index = Self::local_position_to_height_map_index(x & 15, z & 15);
        self.flat_surface_height_map[index] as i32 + 1
    }

    #[must_use]
    pub const fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let index = Self::local_position_to_height_map_index(x & 15, z & 15);
        self.flat_ocean_floor_height_map[index] as i32 + 1
    }

    #[must_use]
    pub const fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let index = Self::local_position_to_height_map_index(x & 15, z & 15);
        self.flat_motion_blocking_height_map[index] as i32 + 1
    }

    #[must_use]
    pub const fn top_motion_blocking_block_no_leaves_height_exclusive(
        &self,
        x: i32,
        z: i32,
    ) -> i32 {
        let index = Self::local_position_to_height_map_index(x & 15, z & 15);
        self.flat_motion_blocking_no_leaves_height_map[index] as i32 + 1
    }

    #[inline]
    const fn local_position_to_height_map_index(x: i32, z: i32) -> usize {
        x as usize * CHUNK_DIM as usize + z as usize
    }

    #[inline]
    const fn local_pos_to_block_index(&self, x: i32, y: i32, z: i32) -> usize {
        self.height() as usize * CHUNK_DIM as usize * x as usize
            + CHUNK_DIM as usize * y as usize
            + z as usize
    }

    #[inline]
    #[must_use]
    pub const fn local_biome_pos_to_biome_index(&self, x: i32, y: i32, z: i32) -> usize {
        let biome_height = self.height() as usize >> 2;
        biome_height * biome_coords::from_block(CHUNK_DIM as i32) as usize * x as usize
            + biome_coords::from_block(CHUNK_DIM as i32) as usize * y as usize
            + z as usize
    }

    #[inline]
    #[must_use]
    pub fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        is_air(self.get_block_state(local_pos))
    }

    #[inline]
    #[must_use]
    pub fn get_block_state_raw(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        let index = self.local_pos_to_block_index(x, y, z);
        self.flat_block_map[index]
    }

    #[inline]
    #[must_use]
    pub fn get_block_state(&self, local_pos: &Vector3<i32>) -> BlockStateId {
        let local_y = local_pos.y - self.bottom_y() as i32;
        if local_y < 0 || local_y >= self.height() as i32 {
            return Block::VOID_AIR.default_state.id;
        }
        self.get_block_state_raw(local_pos.x & 15, local_y, local_pos.z & 15)
    }

    pub fn set_block_state(&mut self, x: i32, y: i32, z: i32, block_state: &BlockState) {
        let local_x = x & 15;
        let local_y = y - self.bottom_y() as i32;
        let local_z = z & 15;

        if local_y < 0 || local_y >= self.height() as i32 {
            return;
        }
        if !block_state.is_air() {
            let index = Self::local_position_to_height_map_index(local_x, local_z);
            let y = y as i16;
            self.maybe_update_surface_height_map(index, y);
            let block = BlockId::from_state_id(block_state.id);

            let blocks_movement = blocks_movement(block_state, block);
            if blocks_movement {
                self.maybe_update_ocean_floor_height_map(index, y);
            }
            if blocks_movement || block_state.is_liquid() {
                self.maybe_update_motion_blocking_height_map(index, y);
                if !block.has_tag(tag::Block::MINECRAFT_LEAVES) {
                    {
                        self.maybe_update_motion_blocking_no_leaves_height_map(index, y);
                    }
                }
            }
        }

        let index = self.local_pos_to_block_index(local_x, local_y, local_z);
        self.flat_block_map[index] = block_state.id;
    }

    #[inline]
    #[must_use]
    pub fn get_biome(&self, x: i32, y: i32, z: i32) -> &'static Biome {
        Biome::from_id(self.get_biome_id(x, y, z)).unwrap_or(&Biome::PLAINS)
    }

    #[inline]
    #[must_use]
    pub fn get_biome_id(&self, x: i32, y: i32, z: i32) -> u8 {
        let index = self.local_biome_pos_to_biome_index(
            x & 3,
            y - biome_coords::from_block(self.bottom_y() as i32),
            z & 3,
        );
        self.flat_biome_map[index]
    }

    pub fn spawn_mobs<T: GenerationCache>(cache: &mut T, block_registry: &dyn WorldPortalExt) {
        let chunk = cache.get_center_chunk();
        if chunk.stage >= StagedChunkEnum::Spawn {
            return;
        }
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Lighting);

        let biome = chunk.get_terrain_gen_biome(
            section_to_block(chunk.x),
            chunk.bottom_y() as i32 + chunk.height() as i32 - 1,
            section_to_block(chunk.z),
        );
        let x = chunk.x;
        let z = chunk.z;

        block_registry.spawn_mobs_for_chunk_generation(cache, biome, x, z);

        let entities = cache
            .get_center_chunk_mut()
            .take_pending_structure_entities();
        block_registry.spawn_structure_entities(entities);

        cache.get_center_chunk_mut().stage = StagedChunkEnum::Spawn;
    }

    #[must_use]
    pub fn get_terrain_gen_biome_id(&self, x: i32, y: i32, z: i32) -> u8 {
        let seed_biome_pos = biome::get_biome_blend(
            self.bottom_y(),
            self.height(),
            self.biome_mixer_seed,
            x,
            y,
            z,
        );

        self.get_biome_id(seed_biome_pos.x, seed_biome_pos.y, seed_biome_pos.z)
    }

    #[must_use]
    pub fn get_terrain_gen_biome(&self, x: i32, y: i32, z: i32) -> &'static Biome {
        Biome::from_id(self.get_terrain_gen_biome_id(x, y, z)).unwrap_or(&Biome::PLAINS)
    }

    pub fn generate_structure_step<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn WorldPortalExt,
        step: usize,
        population_seed: u64,
        world_seed: i64,
    ) {
        let mut tasks = Vec::new();
        {
            let center_chunk = cache.get_center_chunk();
            let center_x = center_chunk.x;
            let center_z = center_chunk.z;

            for (id, instance) in &center_chunk.structure_starts {
                let s = Structure::get(id);
                if s.step.ordinal() != step {
                    continue;
                }

                match instance {
                    StructureInstance::Start(pos) => tasks.push(pos.collector.clone()),
                    StructureInstance::Reference(collector) => {
                        let collector_arc = collector.clone();
                        if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                            tasks.push(collector_arc);
                        }
                    }
                }
            }

            let radius = 8;
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx == 0 && dz == 0 {
                        continue;
                    }

                    let neighbor_x = center_x + dx;
                    let neighbor_z = center_z + dz;

                    if let Some(neighbor) = cache.try_get_proto_chunk(neighbor_x, neighbor_z) {
                        for (id, instance) in &neighbor.structure_starts {
                            let s = Structure::get(id);
                            if s.step.ordinal() != step {
                                continue;
                            }

                            match instance {
                                StructureInstance::Start(pos) => {
                                    let start_x = chunk_pos::start_block_x(center_x);
                                    let start_z = chunk_pos::start_block_z(center_z);
                                    let end_x = start_x + 15;
                                    let end_z = start_z + 15;

                                    if pos
                                        .get_bounding_box()
                                        .intersects_raw_xz(start_x, start_z, end_x, end_z)
                                    {
                                        let collector_arc = pos.collector.clone();
                                        if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                                            tasks.push(collector_arc);
                                        }
                                    }
                                }
                                StructureInstance::Reference(collector) => {
                                    let collector_arc = collector.clone();
                                    if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                                        tasks.push(collector_arc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let decorator_seed = get_decorator_seed(population_seed, 0, step as u64);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(decorator_seed));

        let chunk = cache.get_center_chunk_mut();
        for collector_arc in tasks {
            let mut collector = collector_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            collector.generate_in_chunk(chunk, block_registry, &mut random, world_seed);
        }
    }

    pub fn generate_structures_only<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn WorldPortalExt,
        world_seed: i64,
    ) {
        let (center_x, center_z, min_y) = {
            let chunk = cache.get_center_chunk();
            (chunk.x, chunk.z, chunk.bottom_y() as i32)
        };

        let start_block_x = chunk_pos::start_block_x(center_x);
        let start_block_z = chunk_pos::start_block_z(center_z);
        let population_seed =
            Xoroshiro::get_population_seed(world_seed as u64, start_block_x, start_block_z);

        for step in 0..11 {
            Self::generate_structure_step(cache, block_registry, step, population_seed, world_seed);
        }

        let _ = min_y;
        cache.get_center_chunk_mut().stage = StagedChunkEnum::Features;
    }

    #[must_use]
    pub fn get_allowed_biomes(set: &StructureSet) -> Vec<u16> {
        let mut allowed_biomes = Vec::new();
        for entry in set.structures {
            let structure = Structure::get(&entry.structure);
            if let Some(biomes) = get_tag_ids(
                RegistryKey::WorldgenBiome,
                structure
                    .biomes
                    .strip_prefix('#')
                    .unwrap_or(structure.biomes),
            ) {
                allowed_biomes.extend_from_slice(biomes);
            }
        }
        allowed_biomes
    }

    #[must_use]
    pub const fn start_cell_x(&self, horizontal_cell_block_count: i32) -> i32 {
        self.start_block_x() / horizontal_cell_block_count
    }

    #[must_use]
    pub const fn start_cell_z(&self, horizontal_cell_block_count: i32) -> i32 {
        self.start_block_z() / horizontal_cell_block_count
    }

    #[must_use]
    pub const fn start_block_x(&self) -> i32 {
        start_block_x(self.x)
    }

    #[must_use]
    pub const fn start_block_z(&self) -> i32 {
        start_block_z(self.z)
    }
}

impl BlockAccessor for ProtoChunk {
    fn get_block(&self, position: &BlockPos) -> &'static Block {
        self.get_block_state(&position.0).to_block()
    }

    fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        self.get_block_state(&position.0).to_state()
    }

    fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.get_block_state(&position.0)
    }

    fn get_block_and_state(&self, position: &BlockPos) -> (&'static Block, &'static BlockState) {
        let id = self.get_block_state(&position.0);
        BlockState::from_id_with_block(id)
    }
}

impl BlockPlacer for ProtoChunk {
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        self.get_block_state(pos)
    }

    fn set_block_state(&mut self, pos: &Vector3<i32>, state: &BlockState) {
        Self::set_block_state(self, pos.x, pos.y, pos.z, state);
    }

    fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.add_block_entity(nbt);
    }
}

impl GenerationCache for ProtoChunk {
    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk {
        self
    }
    fn get_center_chunk(&self) -> &ProtoChunk {
        self
    }
    fn get_chunk_mut(&mut self, cx: i32, cz: i32) -> Option<&mut ProtoChunk> {
        (cx == self.x && cz == self.z).then_some(self)
    }
    fn get_chunk(&self, cx: i32, cz: i32) -> Option<&ProtoChunk> {
        (cx == self.x && cz == self.z).then_some(self)
    }
    fn try_get_proto_chunk(&self, cx: i32, cz: i32) -> Option<&ProtoChunk> {
        self.get_chunk(cx, cz)
    }
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        Self::get_block_state(self, pos)
    }
    fn get_fluid_and_fluid_state(&self, _pos: &Vector3<i32>) -> (Fluid, FluidState) {
        (
            Fluid::EMPTY,
            FluidState {
                height: 0.0,
                level: 0,
                is_empty: true,
                blast_resistance: 0.0,
                block_state_id: BlockStateId::AIR,
                is_still: false,
                is_source: false,
                falling: false,
            },
        )
    }
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState) {
        Self::set_block_state(self, pos.x, pos.y, pos.z, block_state);
    }
    fn add_block_entity(&mut self, _pos: &Vector3<i32>, nbt: NbtCompound) {
        self.add_block_entity(nbt);
    }
    fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        Self::top_motion_blocking_block_height_exclusive(self, x, z)
    }
    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32 {
        Self::top_motion_blocking_block_no_leaves_height_exclusive(self, x, z)
    }
    fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32 {
        Self::get_top_y(self, heightmap, x, z)
    }
    fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        Self::top_block_height_exclusive(self, x, z)
    }
    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        Self::ocean_floor_height_exclusive(self, x, z)
    }
    fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        self.is_air(local_pos)
    }
    fn get_biome_for_terrain_gen(&self, x: i32, y: i32, z: i32) -> &'static Biome {
        Self::get_biome(self, x, y, z)
    }
    fn get_blending_data(
        &self,
        _cx: i32,
        _cz: i32,
    ) -> Option<&crate::generation::blender::blending_data::BlendingData> {
        None
    }
}
