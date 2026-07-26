use pumpkin_data::block_properties::is_air;
use pumpkin_data::chunk::DoublePerlinNoiseParameters;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::structures::StructureKeys;
use pumpkin_data::{Block, BlockState, block_properties::blocks_movement, chunk::Biome};
use pumpkin_data::{BlockId, BlockStateId, tag};
use pumpkin_util::{
    HeightMap,
    math::{position::BlockPos, vector3::Vector3},
};
use rustc_hash::FxHashMap;

use super::{
    GlobalRandomConfig, biome_coords,
    noise::router::proto_noise_router::DoublePerlinNoiseBuilder,
    positions::chunk_pos::{start_block_x, start_block_z},
    surface::terrain::SurfaceTerrainBuilder,
};
use crate::biome::{MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::chunk::format::LightContainer;
use crate::chunk::{ChunkData, ChunkHeightmapType, ChunkLight};
use crate::chunk_system::StagedChunkEnum;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::noise::CHUNK_DIM;
use crate::generation::noise::aquifer_sampler::{FluidLevel, FluidLevelSamplerImpl};
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::structure::structures::StructureInstance;
use crate::{chunk::CHUNK_AREA, world::BlockAccessor};
use pumpkin_nbt::compound::NbtCompound;

use crate::tick::{ScheduledTick, TickPriority};

mod steps;
mod structures;

enum ActiveSupplier {
    Overworld(MultiNoiseBiomeSupplier),
    Nether(MultiNoiseBiomeSupplier),
    End(TheEndBiomeSupplier),
}

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
    flat_ocean_floor_height_map: [i16; CHUNK_AREA],
    pub flat_motion_blocking_height_map: [i16; CHUNK_AREA],
    pub flat_motion_blocking_no_leaves_height_map: [i16; CHUNK_AREA],
    structure_starts: FxHashMap<StructureKeys, StructureInstance>,

    height: u16,
    bottom_y: i8,
    pub stage: StagedChunkEnum,
    pub light: ChunkLight,
    pub carving_mask: crate::generation::carver::mask::CarvingMask,
    pub blending_data: Option<crate::generation::blender::blending_data::BlendingData>,
    pub pending_block_entities: Vec<NbtCompound>,
    /// Entities from structure templates (villagers, golems, animals, etc.).
    /// Consumed when the proto-chunk is finalized into a level chunk, then
    /// spawned on first entity-chunk load.
    pub pending_entities: Vec<NbtCompound>,
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
    #[must_use]
    pub fn new(x: i32, z: i32, generator: &super::generator::WorldGenerator) -> Self {
        let dimension = generator.dimension();
        // Chunk storage covers the dimension's complete physical build range.  In
        // particular, the Nether has a 256-block physical height but a 128-block
        // logical height used only for portal and entity limits.
        let height = dimension.height as u16;
        let section_count = (height as usize) / 16;

        let default_block = match generator {
            super::generator::WorldGenerator::Noise(noise_gen) => noise_gen.default_block,
            super::generator::WorldGenerator::Flat(_) => Block::AIR.default_state,
        };
        let biome_mixer_seed = match generator {
            super::generator::WorldGenerator::Noise(noise_gen) => noise_gen.biome_mixer_seed,
            super::generator::WorldGenerator::Flat(flat_gen) => {
                crate::biome::hash_seed(flat_gen.seed)
            }
        };

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
            bottom_y: dimension.min_y as i8,
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
                dimension.min_y,
            ),
            blending_data: None,
            pending_block_entities: Vec::new(),
            pending_entities: Vec::new(),
            fluid_ticks: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_chunk_data(
        chunk_data: &ChunkData,
        generator: &super::generator::WorldGenerator,
    ) -> Self {
        let mut proto_chunk = Self::new(chunk_data.x, chunk_data.z, generator);

        proto_chunk.light = chunk_data.light_engine.lock().unwrap().clone();
        proto_chunk
            .blending_data
            .clone_from(&chunk_data.blending_data);

        let section_data = &chunk_data.section;
        let heightmap_data = chunk_data.heightmap.lock().unwrap();

        let block_sections_guard = section_data.block_sections.read().unwrap();
        let biome_sections_guard = section_data.biome_sections.read().unwrap();

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
                let index = ((z << 4) + x) as usize;

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

        proto_chunk.stage = StagedChunkEnum::from(chunk_data.status);
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

    pub fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.pending_block_entities.push(nbt);
    }

    pub fn take_pending_block_entities(&mut self) -> Vec<NbtCompound> {
        std::mem::take(&mut self.pending_block_entities)
    }

    /// Queue a structure-template entity (villager, iron golem, …) for this chunk.
    pub fn add_entity(&mut self, nbt: NbtCompound) {
        self.pending_entities.push(nbt);
    }

    pub fn take_pending_entities(&mut self) -> Vec<NbtCompound> {
        std::mem::take(&mut self.pending_entities)
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
        self.flat_surface_height_map[index] = current_height.max(y) as _;
    }

    fn maybe_update_ocean_floor_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_ocean_floor_height_map[index];
        self.flat_ocean_floor_height_map[index] = current_height.max(y) as _;
    }

    fn maybe_update_motion_blocking_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_motion_blocking_height_map[index];
        self.flat_motion_blocking_height_map[index] = current_height.max(y) as _;
    }

    fn maybe_update_motion_blocking_no_leaves_height_map(&mut self, index: usize, y: i16) {
        let current_height = self.flat_motion_blocking_no_leaves_height_map[index];
        self.flat_motion_blocking_no_leaves_height_map[index] = current_height.max(y) as _;
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
        Biome::from_id(self.get_biome_id(x, y, z)).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn get_biome_id(&self, x: i32, y: i32, z: i32) -> u8 {
        let index = self.local_biome_pos_to_biome_index(
            x & biome_coords::from_block(15),
            y - biome_coords::from_block(self.bottom_y() as i32),
            z & biome_coords::from_block(15),
        );
        self.flat_biome_map[index]
    }

    const fn start_cell_x(&self, horizontal_cell_block_count: i32) -> i32 {
        self.start_block_x() / horizontal_cell_block_count
    }

    const fn start_cell_z(&self, horizontal_cell_block_count: i32) -> i32 {
        self.start_block_z() / horizontal_cell_block_count
    }

    const fn start_block_x(&self) -> i32 {
        start_block_x(self.x)
    }

    const fn start_block_z(&self) -> i32 {
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

#[cfg(test)]
mod tests {
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::structures::StructureSet;
    use pumpkin_util::world_seed::Seed;

    use crate::generation::generator::VanillaGenerator;
    use crate::generation::get_world_gen;

    use super::{
        Block, FluidLevel, FluidLevelSamplerImpl, HeightMap, ProtoChunk,
        StandardChunkFluidLevelSampler, Vector3,
    };

    fn overworld_chunk() -> ProtoChunk {
        let world_gen = get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        ProtoChunk::new(0, 0, &world_gen)
    }

    #[test]
    fn out_of_bounds_positions_read_as_void_air() {
        let chunk = overworld_chunk();
        assert_eq!(
            chunk.get_block_state(&Vector3::new(0, 1000, 0)),
            Block::VOID_AIR.default_state.id
        );
        assert_eq!(
            chunk.get_block_state(&Vector3::new(0, -1000, 0)),
            Block::VOID_AIR.default_state.id
        );
    }

    #[test]
    fn set_block_state_updates_heightmaps() {
        let mut chunk = overworld_chunk();
        let default_top = chunk.top_block_height_exclusive(5, 7);

        chunk.set_block_state(5, 10, 7, Block::STONE.default_state);

        assert_eq!(
            chunk.get_block_state(&Vector3::new(5, 10, 7)),
            Block::STONE.default_state.id
        );
        assert_eq!(chunk.top_block_height_exclusive(5, 7), 11);
        assert_eq!(chunk.ocean_floor_height_exclusive(5, 7), 11);
        assert_eq!(chunk.top_motion_blocking_block_height_exclusive(5, 7), 11);
        assert_eq!(
            chunk.top_motion_blocking_block_no_leaves_height_exclusive(5, 7),
            11
        );
        assert!(default_top < 11);

        // Air placements do not raise the height maps
        chunk.set_block_state(5, 20, 7, Block::AIR.default_state);
        assert_eq!(chunk.top_block_height_exclusive(5, 7), 11);
    }

    #[test]
    fn get_top_y_dispatches_to_heightmaps() {
        let mut chunk = overworld_chunk();
        chunk.set_block_state(1, 33, 2, Block::STONE.default_state);

        assert_eq!(
            chunk.get_top_y(&HeightMap::WorldSurface, 1, 2),
            chunk.top_block_height_exclusive(1, 2)
        );
        assert_eq!(
            chunk.get_top_y(&HeightMap::OceanFloor, 1, 2),
            chunk.ocean_floor_height_exclusive(1, 2)
        );
        assert_eq!(
            chunk.get_top_y(&HeightMap::MotionBlocking, 1, 2),
            chunk.top_motion_blocking_block_height_exclusive(1, 2)
        );
        assert_eq!(
            chunk.get_top_y(&HeightMap::MotionBlockingNoLeaves, 1, 2),
            chunk.top_motion_blocking_block_no_leaves_height_exclusive(1, 2)
        );
    }

    #[test]
    fn fluid_level_sampler_switches_at_bottom_fluid() {
        let sampler = StandardChunkFluidLevelSampler::new(
            FluidLevel::new(63, &Block::WATER),
            FluidLevel::new(-54, &Block::LAVA),
        );

        assert_eq!(sampler.get_fluid_level(0, -60, 0).max_y_exclusive(), -54);
        assert_eq!(sampler.get_fluid_level(0, 0, 0).max_y_exclusive(), 63);
        assert_eq!(sampler.get_fluid_level(0, 200, 0).max_y_exclusive(), 63);
    }

    #[test]
    fn moved_generation_apis_remain_reachable() {
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::step_to_biomes;
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::step_to_noise;
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::step_to_surface;
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::step_to_carvers;
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::set_structure_starts;
        let _: fn(&mut ProtoChunk, &VanillaGenerator) = ProtoChunk::set_structure_references;
        let _: fn(&StructureSet) -> Vec<u16> = ProtoChunk::get_allowed_biomes;
    }
}
