use std::collections::HashMap;
use std::ptr;
use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::tag;
use pumpkin_data::{
    Block, BlockState, block_properties::blocks_movement, chunk::Biome, tag::Taggable,
};
use pumpkin_util::{
    HeightMap,
    math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
    random::{RandomGenerator, get_decorator_seed, xoroshiro128::Xoroshiro},
};

use super::{
    GlobalRandomConfig,
    aquifer_sampler::{FluidLevel, FluidLevelSamplerImpl},
    biome_coords,
    chunk_noise::{CHUNK_DIM, ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK},
    feature::placed_features::PLACED_FEATURES,
    noise::router::{
        multi_noise_sampler::MultiNoiseSampler, proto_noise_router::DoublePerlinNoiseBuilder,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
    positions::chunk_pos::{start_block_x, start_block_z},
    section_coords,
    settings::GenerationSettings,
    surface::{MaterialRuleContext, estimate_surface_height, terrain::SurfaceTerrainBuilder},
};
use crate::chunk::format::LightContainer;
use crate::chunk::palette::{BiomePalette, BlockPalette};
use crate::chunk::{ChunkData, ChunkLight, ChunkSections, SubChunk};
use crate::generation::height_limit::HeightLimitView;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::structure::placement::StructurePlacementCalculator;
use crate::generation::structure::structures::StructurePosition;
use crate::generation::structure::{STRUCTURE_SETS, STRUCTURES, Structure, StructureType};
use crate::level::new_chunk_system::StagedChunkEnum;
use crate::{
    BlockStateId,
    biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier},
    block::RawBlockState,
    chunk::CHUNK_AREA,
    dimension::Dimension,
    generation::{biome, positions::chunk_pos},
    level::Level,
    world::{BlockAccessor, BlockRegistryExt},
};

pub trait GenerationCache: HeightLimitView + BlockAccessor {
    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk;
    fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState;
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState);
    fn top_motion_blocking_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32;
    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, pos: &Vector2<i32>) -> i32;
    fn get_top_y(&self, heightmap: &HeightMap, pos: &Vector2<i32>) -> i32;
    fn top_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32;
    fn ocean_floor_height_exclusive(&self, pos: &Vector2<i32>) -> i32;
    fn is_air(&self, local_pos: &Vector3<i32>) -> bool;
    fn get_biome_for_terrain_gen(&self, global_block_pos: &Vector3<i32>) -> &'static Biome;
}

const AIR_BLOCK: Block = Block::AIR;

pub struct StandardChunkFluidLevelSampler {
    top_fluid: FluidLevel,
    bottom_fluid: FluidLevel,
    bottom_y: i32,
}

impl StandardChunkFluidLevelSampler {
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
    fn get_fluid_level(&self, _x: i32, y: i32, _z: i32) -> FluidLevel {
        if y < self.bottom_y {
            self.bottom_fluid.clone()
        } else {
            self.top_fluid.clone()
        }
    }
}

/// Vanilla Chunk Steps
///
/// 1. empty: The chunk is not yet loaded or generated.
///
/// 2. structures_starts: This step calculates the starting points for structure pieces. For structures that are the starting in this chunk, the position of all pieces are generated and stored.
///
/// 3. structures_references: A reference to nearby chunks that have a structures' starting point are stored.
///
/// 4. biomes: Biomes are determined and stored. No terrain is generated at this stage.
///
/// 5. noise: The base terrain shape and liquid bodies are placed.
///
/// 6. surface: The surface of the terrain is replaced with biome-dependent blocks.
///
/// 7. carvers: Carvers carve certain parts of the terrain and replace solid blocks with air.
///
/// 8. features: Features and structure pieces are placed and heightmaps are generated.
///
/// 9. initialize_light: The lighting engine is initialized and light sources are identified.
///
/// 10. light: The lighting engine calculates the light level for blocks.
///
/// 11. spawn: Mobs are spawned.
///
/// 12. full: Generation is done and a chunk can now be loaded. The proto-chunk is now converted to a level chunk and all block updates deferred in the above steps are executed.
///
#[derive(Debug)]
pub struct ProtoChunk {
    pub chunk_pos: Vector2<i32>,
    pub default_block: &'static BlockState,
    biome_mixer_seed: i64,
    // These are local positions
    flat_block_map: Box<[BlockStateId]>,
    flat_biome_map: Box<[&'static Biome]>,
    /// HEIGHTMAPS
    ///
    /// Top block that is not air
    pub flat_surface_height_map: Box<[i16]>,
    flat_ocean_floor_height_map: Box<[i16]>,
    pub flat_motion_blocking_height_map: Box<[i16]>,
    pub flat_motion_blocking_no_leaves_height_map: Box<[i16]>,
    // may want to use chunk status
    structure_starts: HashMap<Structure, (StructurePosition, StructureType)>,
    // Height of the chunk for indexing
    height: u16,
    bottom_y: i8,
}

pub struct TerrainCache {
    pub terrain_builder: SurfaceTerrainBuilder,
    pub surface_noise: DoublePerlinNoiseSampler,
    pub secondary_noise: DoublePerlinNoiseSampler,
}

impl TerrainCache {
    pub fn from_random(random_config: &GlobalRandomConfig) -> Self {
        let random = &random_config.base_random_deriver;
        let mut noise_builder = DoublePerlinNoiseBuilder::new(random_config);
        let terrain_builder = SurfaceTerrainBuilder::new(&mut noise_builder, random);
        let surface_noise = noise_builder.get_noise_sampler_for_id("surface");
        let secondary_noise = noise_builder.get_noise_sampler_for_id("surface_secondary");
        Self {
            terrain_builder,
            surface_noise,
            secondary_noise,
        }
    }
}

impl ProtoChunk {
    pub fn new(
        chunk_pos: Vector2<i32>,
        settings: &GenerationSettings,
        default_block: &'static BlockState,
        biome_mixer_seed: i64,
    ) -> Self {
        let generation_shape = &settings.shape;
        let height = generation_shape.height;

        let default_heightmap = vec![i16::MIN; CHUNK_AREA].into_boxed_slice();
        Self {
            chunk_pos,
            default_block,
            flat_block_map: vec![0; CHUNK_AREA * height as usize].into_boxed_slice(),
            flat_biome_map: vec![
                &Biome::PLAINS;
                biome_coords::from_block(CHUNK_DIM as usize)
                    * biome_coords::from_block(CHUNK_DIM as usize)
                    * biome_coords::from_block(height as usize)
            ]
            .into_boxed_slice(),
            biome_mixer_seed,
            flat_surface_height_map: default_heightmap.clone(),
            flat_ocean_floor_height_map: default_heightmap.clone(),
            flat_motion_blocking_height_map: default_heightmap.clone(),
            flat_motion_blocking_no_leaves_height_map: default_heightmap,
            structure_starts: HashMap::new(),
            height,
            bottom_y: generation_shape.min_y,
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn bottom_y(&self) -> i8 {
        self.bottom_y
    }

    fn maybe_update_surface_height_map(&mut self, pos: &Vector3<i32>) {
        let local_x = pos.x & 15;
        let local_z = pos.z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        let current_height = self.flat_surface_height_map[index];

        if pos.y > current_height as i32 {
            self.flat_surface_height_map[index] = pos.y as _;
        }
    }

    fn maybe_update_ocean_floor_height_map(&mut self, pos: &Vector3<i32>) {
        let local_x = pos.x & 15;
        let local_z = pos.z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        let current_height = self.flat_ocean_floor_height_map[index];

        if pos.y > current_height as i32 {
            self.flat_ocean_floor_height_map[index] = pos.y as _;
        }
    }

    fn maybe_update_motion_blocking_height_map(&mut self, pos: &Vector3<i32>) {
        let local_x = pos.x & 15;
        let local_z = pos.z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        let current_height = self.flat_motion_blocking_height_map[index];

        if pos.y > current_height as i32 {
            self.flat_motion_blocking_height_map[index] = pos.y as _;
        }
    }

    fn maybe_update_motion_blocking_no_leaves_height_map(&mut self, pos: &Vector3<i32>) {
        let local_x = pos.x & 15;
        let local_z = pos.z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        let current_height = self.flat_motion_blocking_no_leaves_height_map[index];

        if pos.y > current_height as i32 {
            self.flat_motion_blocking_no_leaves_height_map[index] = pos.y as _;
        }
    }

    pub fn get_top_y(&self, heightmap: &HeightMap, pos: &Vector2<i32>) -> i32 {
        match heightmap {
            HeightMap::WorldSurfaceWg => self.top_block_height_exclusive(pos),
            HeightMap::WorldSurface => self.top_block_height_exclusive(pos),
            HeightMap::OceanFloorWg => self.ocean_floor_height_exclusive(pos),
            HeightMap::OceanFloor => self.ocean_floor_height_exclusive(pos),
            HeightMap::MotionBlocking => self.top_motion_blocking_block_height_exclusive(pos),
            HeightMap::MotionBlockingNoLeaves => {
                self.top_motion_blocking_block_no_leaves_height_exclusive(pos)
            }
        }
    }

    pub fn top_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let local_x = pos.x & 15;
        let local_z = pos.y & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_surface_height_map[index] as i32 + 1
    }

    pub fn ocean_floor_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let local_x = pos.x & 15;
        let local_z = pos.y & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_ocean_floor_height_map[index] as i32 + 1
    }

    pub fn top_motion_blocking_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let local_x = pos.x & 15;
        let local_z = pos.y & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_motion_blocking_height_map[index] as i32 + 1
    }

    pub fn top_motion_blocking_block_no_leaves_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let local_x = pos.x & 15;
        let local_z = pos.y & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_motion_blocking_no_leaves_height_map[index] as i32 + 1
    }

    #[inline]
    fn local_position_to_height_map_index(x: i32, z: i32) -> usize {
        x as usize * CHUNK_DIM as usize + z as usize
    }

    #[inline]
    fn local_pos_to_block_index(&self, local_pos: &Vector3<i32>) -> usize {
        #[cfg(debug_assertions)]
        {
            assert!(local_pos.x >= 0 && local_pos.x <= 15);
            assert!(local_pos.y < self.height() as i32);
            assert!(local_pos.y >= 0);
            assert!(local_pos.z >= 0 && local_pos.z <= 15);
        }
        self.height() as usize * CHUNK_DIM as usize * local_pos.x as usize
            + CHUNK_DIM as usize * local_pos.y as usize
            + local_pos.z as usize
    }

    #[inline]
    fn local_biome_pos_to_biome_index(&self, local_biome_pos: &Vector3<i32>) -> usize {
        #[cfg(debug_assertions)]
        {
            assert!(local_biome_pos.x >= 0 && local_biome_pos.x <= 3);
            assert!(
                local_biome_pos.y < biome_coords::from_chunk(self.height() as i32)
                    && local_biome_pos.y >= 0,
                "{} - {} vs {}",
                0,
                biome_coords::from_chunk(self.height() as i32),
                local_biome_pos.y
            );
            assert!(local_biome_pos.z >= 0 && local_biome_pos.z <= 3);
        }

        biome_coords::from_block(self.height() as usize)
            * biome_coords::from_block(CHUNK_DIM as usize)
            * local_biome_pos.x as usize
            + biome_coords::from_block(CHUNK_DIM as usize) * local_biome_pos.y as usize
            + local_biome_pos.z as usize
    }

    #[inline]
    pub fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        let state = self.get_block_state(local_pos).to_state();
        state.is_air()
    }

    #[inline]
    pub fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState {
        let local_pos = Vector3::new(pos.x & 15, pos.y - self.bottom_y() as i32, pos.z & 15);
        if local_pos.y < 0 || local_pos.y >= self.height() as i32 {
            return RawBlockState(Block::VOID_AIR.default_state.id);
        }
        let index = self.local_pos_to_block_index(&local_pos);
        RawBlockState(self.flat_block_map[index])
    }

    pub fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState) {
        let local_pos = Vector3::new(pos.x & 15, pos.y - self.bottom_y() as i32, pos.z & 15);
        if local_pos.y < 0 || local_pos.y >= self.height() as i32 {
            return;
        }
        if !block_state.is_air() {
            self.maybe_update_surface_height_map(pos);
        }

        if blocks_movement(block_state) {
            self.maybe_update_ocean_floor_height_map(pos);
        }

        if blocks_movement(block_state) || block_state.is_liquid() {
            self.maybe_update_motion_blocking_height_map(pos);
            let block = Block::from_state_id(block_state.id);
            if !block.is_tagged_with_by_tag(&tag::Block::MINECRAFT_LEAVES) {
                {
                    self.maybe_update_motion_blocking_no_leaves_height_map(pos);
                }
            }
        }

        let index = self.local_pos_to_block_index(&local_pos);
        self.flat_block_map[index] = block_state.id;
    }

    #[inline]
    pub fn get_biome(&self, global_biome_pos: &Vector3<i32>) -> &'static Biome {
        let local_pos = Vector3::new(
            global_biome_pos.x & biome_coords::from_block(15),
            global_biome_pos.y - biome_coords::from_block(self.bottom_y() as i32),
            global_biome_pos.z & biome_coords::from_block(15),
        );
        let index = self.local_biome_pos_to_biome_index(&local_pos);
        self.flat_biome_map[index]
    }

    pub fn populate_biomes(
        &mut self,
        dimension: Dimension,
        multi_noise_sampler: &mut MultiNoiseSampler,
    ) {
        let min_y = self.bottom_y();
        let bottom_section = section_coords::block_to_section(min_y) as i32;
        let top_section = section_coords::block_to_section(min_y as i32 + self.height() as i32 - 1);

        let start_block_x = chunk_pos::start_block_x(&self.chunk_pos);
        let start_block_z = chunk_pos::start_block_z(&self.chunk_pos);

        let start_biome_x = biome_coords::from_block(start_block_x);
        let start_biome_z = biome_coords::from_block(start_block_z);

        for i in bottom_section..=top_section {
            let start_block_y = section_coords::section_to_block(i);
            let start_biome_y = biome_coords::from_block(start_block_y);

            let biomes_per_section = biome_coords::from_block(CHUNK_DIM) as i32;
            for x in 0..biomes_per_section {
                for y in 0..biomes_per_section {
                    for z in 0..biomes_per_section {
                        let biome_pos =
                            Vector3::new(start_biome_x + x, start_biome_y + y, start_biome_z + z);
                        let biome = if dimension == Dimension::End {
                            TheEndBiomeSupplier::biome(&biome_pos, multi_noise_sampler, dimension)
                        } else {
                            MultiNoiseBiomeSupplier::biome(
                                &biome_pos,
                                multi_noise_sampler,
                                dimension,
                            )
                        };
                        //dbg!("Populating biome: {:?} -> {:?}", biome_pos, biome);

                        let local_biome_pos = Vector3 {
                            x,
                            // Make the y start from 0
                            y: start_biome_y + y - biome_coords::from_block(min_y as i32),
                            z,
                        };
                        let index = self.local_biome_pos_to_biome_index(&local_biome_pos);

                        self.flat_biome_map[index] = biome;
                    }
                }
            }
        }
    }

    pub fn populate_noise(
        &mut self,
        noise_sampler: &mut ChunkNoiseGenerator,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let horizontal_cell_block_count = noise_sampler.horizontal_cell_block_count();
        let vertical_cell_block_count = noise_sampler.vertical_cell_block_count();
        let horizontal_cells = CHUNK_DIM / horizontal_cell_block_count;

        let min_y = self.bottom_y();
        let minimum_cell_y = min_y / vertical_cell_block_count as i8;
        let cell_height = self.height() / vertical_cell_block_count as u16;

        let start_block_x = self.start_block_x();
        let start_block_z = self.start_block_z();
        let start_cell_x = self.start_cell_x(horizontal_cell_block_count);
        let start_cell_z = self.start_cell_z(horizontal_cell_block_count);

        // TODO: Block state updates when we implement those
        noise_sampler.sample_start_density();
        for cell_x in 0..horizontal_cells {
            noise_sampler.sample_end_density(cell_x);
            let sample_start_x =
                (start_cell_x + cell_x as i32) * horizontal_cell_block_count as i32;

            for cell_z in 0..horizontal_cells {
                let sample_start_z =
                    (start_cell_z + cell_z as i32) * horizontal_cell_block_count as i32;

                for cell_y in (0..cell_height).rev() {
                    noise_sampler.on_sampled_cell_corners(cell_x, cell_y, cell_z);
                    let sample_start_y =
                        (minimum_cell_y as i32 + cell_y as i32) * vertical_cell_block_count as i32;

                    let block_y_base = sample_start_y;
                    let delta_y_step = 1.0 / vertical_cell_block_count as f64;

                    for local_y in (0..vertical_cell_block_count).rev() {
                        let block_y = block_y_base + local_y as i32;
                        let delta_y = local_y as f64 * delta_y_step;
                        noise_sampler.interpolate_y(delta_y);

                        let block_x_base =
                            start_block_x + cell_x as i32 * horizontal_cell_block_count as i32;
                        let delta_x_step = 1.0 / horizontal_cell_block_count as f64;

                        for local_x in 0..horizontal_cell_block_count {
                            let block_x = block_x_base + local_x as i32;
                            let delta_x = local_x as f64 * delta_x_step;
                            noise_sampler.interpolate_x(delta_x);

                            let block_z_base =
                                start_block_z + cell_z as i32 * horizontal_cell_block_count as i32;
                            let delta_z_step = 1.0 / horizontal_cell_block_count as f64;

                            for local_z in 0..horizontal_cell_block_count {
                                let block_z = block_z_base + local_z as i32;
                                let delta_z = local_z as f64 * delta_z_step;
                                noise_sampler.interpolate_z(delta_z);

                                // The `cell_offset` calculations are still a good idea for clarity and correctness
                                // but let's confirm the values.
                                // block_x = start_block_x + cell_x*H + local_x
                                // sample_start_x = start_cell_x*H + cell_x*H = (start_cell_x+cell_x)*H
                                // These can be simplified.
                                let cell_offset_x = local_x as i32;
                                let cell_offset_y = block_y - sample_start_y;
                                let cell_offset_z = local_z as i32;

                                let block_state = noise_sampler
                                    .sample_block_state(
                                        Vector3::new(
                                            sample_start_x,
                                            sample_start_y,
                                            sample_start_z,
                                        ),
                                        Vector3::new(cell_offset_x, cell_offset_y, cell_offset_z),
                                        surface_height_estimate_sampler,
                                    )
                                    .unwrap_or(self.default_block);
                                self.set_block_state(
                                    &Vector3::new(block_x, block_y, block_z),
                                    block_state,
                                );
                            }
                        }
                    }
                }
            }
            noise_sampler.swap_buffers();
        }
    }

    pub fn get_biome_for_terrain_gen(&self, global_block_pos: &Vector3<i32>) -> &'static Biome {
        let seed_biome_pos = biome::get_biome_blend(
            self.bottom_y(),
            self.height(),
            self.biome_mixer_seed,
            global_block_pos,
        );

        self.get_biome(&seed_biome_pos)
    }

    /// Constructs the terrain surface, although "surface" is a misnomer as it also places underground blocks like bedrock and deepslate.
    /// This stage also generates larger decorative structures, such as badlands pillars and icebergs.
    ///
    /// It is crucial that biome assignments are determined before this process begins.
    pub fn build_surface(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let start_x = chunk_pos::start_block_x(&self.chunk_pos);
        let start_z = chunk_pos::start_block_z(&self.chunk_pos);
        let min_y = self.bottom_y();

        let random = &random_config.base_random_deriver;
        let noise_builder = DoublePerlinNoiseBuilder::new(random_config);
        let mut context = MaterialRuleContext::new(
            min_y,
            self.height(),
            noise_builder,
            random,
            &terrain_cache.terrain_builder,
            &terrain_cache.surface_noise,
            &terrain_cache.secondary_noise,
            settings.sea_level,
        );
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = start_x + local_x;
                let z = start_z + local_z;

                let mut top_block =
                    self.top_block_height_exclusive(&Vector2::new(local_x, local_z));

                let biome_y = if settings.legacy_random_source {
                    0
                } else {
                    top_block
                };

                let this_biome = self.get_biome_for_terrain_gen(&Vector3::new(x, biome_y, z));
                if this_biome == &Biome::ERODED_BADLANDS {
                    terrain_cache
                        .terrain_builder
                        .place_badlands_pillar(self, x, z, top_block);
                    // Get the top block again if we placed a pillar!

                    top_block = self.top_block_height_exclusive(&Vector2::new(local_x, local_z));
                }

                context.init_horizontal(x, z);

                let mut stone_depth_above = 0;
                let mut min = i32::MAX;
                let mut fluid_height = i32::MIN;
                for y in (min_y as i32..top_block).rev() {
                    let pos = Vector3::new(x, y, z);
                    let state = self.get_block_state(&pos).to_state();
                    if state.is_air() {
                        stone_depth_above = 0;
                        fluid_height = i32::MIN;
                        continue;
                    }
                    if state.is_liquid() {
                        if fluid_height == i32::MIN {
                            fluid_height = y + 1;
                        }
                        continue;
                    }
                    if min >= y {
                        let shift = min_y << 4;
                        min = shift as i32;

                        for search_y in (min_y as i32 - 1..=y - 1).rev() {
                            if search_y < min_y as i32 {
                                min = search_y + 1;
                                break;
                            }

                            let state = self
                                .get_block_state(&Vector3::new(local_x, search_y, local_z))
                                .to_block();

                            // TODO: Is there a better way to check that its not a fluid?
                            if !(state != &AIR_BLOCK
                                && state != &WATER_BLOCK
                                && state != &LAVA_BLOCK)
                            {
                                min = search_y + 1;
                                break;
                            }
                        }
                    }

                    // let biome_pos = Vector3::new(x, biome_y as i32, z);
                    stone_depth_above += 1;
                    let stone_depth_below = y - min + 1;
                    context.init_vertical(stone_depth_above, stone_depth_below, y, fluid_height);
                    // panic!("Blending with biome {:?} at: {:?}", biome, biome_pos);

                    if state.id == self.default_block.id {
                        context.biome = self.get_biome_for_terrain_gen(&context.block_pos);
                        let new_state = settings.surface_rule.try_apply(
                            self,
                            &mut context,
                            surface_height_estimate_sampler,
                        );

                        if let Some(state) = new_state {
                            self.set_block_state(&pos, state);
                        }
                    }
                }
                if this_biome == &Biome::FROZEN_OCEAN || this_biome == &Biome::DEEP_FROZEN_OCEAN {
                    let surface_estimate =
                        estimate_surface_height(&mut context, surface_height_estimate_sampler);

                    terrain_cache.terrain_builder.place_iceberg(
                        self,
                        this_biome,
                        x,
                        z,
                        surface_estimate,
                        top_block,
                        settings.sea_level,
                        &random_config.base_random_deriver,
                    );
                }
            }
        }
    }

    /// This generates "Structure Pieces" and "Features" also known as decorations, which include things like trees, grass, ores, and more.
    /// Essentially, it encompasses everything above the surface or underground. It's crucial that this step is executed after biomes are generated,
    /// as the decoration directly depends on the biome. Similarly, running this after the surface is built is logical, as it often involves checking block types.
    /// For example, flowers are typically placed only on grass blocks.
    ///
    /// Features are defined across two separate asset files, each serving a distinct purpose:
    ///
    /// 1. First, we determine **whether** to generate a feature and **at which block positions** to place it.
    /// 2. Then, using the second file, we determine **how** to generate the feature.
    pub fn generate_features_and_structure<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn BlockRegistryExt,
        random_config: &GlobalRandomConfig,
    ) {
        let chunk = cache.get_center_chunk_mut();
        let chunk_pos = chunk.chunk_pos;
        let min_y = chunk.bottom_y();
        let height = chunk.height();

        let bottom_section = section_coords::block_to_section(min_y) as i32;
        let block_pos = BlockPos(Vector3::new(
            section_coords::section_to_block(chunk_pos.x),
            bottom_section,
            section_coords::section_to_block(chunk_pos.y),
        ));

        let population_seed =
            Xoroshiro::get_population_seed(random_config.seed, block_pos.0.x, block_pos.0.z);

        for (_structure, (pos, stype)) in chunk.structure_starts.clone() {
            dbg!("generating structure");
            stype.generate(pos.clone(), chunk);
        }

        // TODO: This needs to be different depending on what biomes are in the chunk -> affects the
        // random
        for (name, feature) in PLACED_FEATURES.iter() {
            // TODO: Properly set index and step
            let decorator_seed = get_decorator_seed(population_seed, 0, 0);
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(decorator_seed));
            feature.generate(
                cache,
                block_registry,
                min_y,
                height,
                name,
                &mut random,
                block_pos,
            );
        }
    }

    pub fn set_structure_starts(&mut self, random_config: &GlobalRandomConfig) {
        for (name, set) in STRUCTURE_SETS.iter() {
            let calculator = StructurePlacementCalculator {
                seed: random_config.seed as i64,
            };
            // for structure in &set.structures {
            //     let start = self.structure_starts.get(STRUCTURES.get(name).unwrap());
            // }
            if !set.placement.should_generate(calculator, self.chunk_pos) {
                continue; // ??
            }

            if set.structures.len() == 1 {
                let position = set.structures[0]
                    .structure
                    .get_structure_position(name, self);
                if let Some(position) = position
                    && !position.generator.pieces_positions.is_empty()
                {
                    self.structure_starts.insert(
                        STRUCTURES.get(name).unwrap().clone(),
                        (position, set.structures[0].structure.clone()),
                    );
                }
                return;
            }
            // TODO: handle multiple structures
        }
    }

    fn start_cell_x(&self, horizontal_cell_block_count: u8) -> i32 {
        self.start_block_x() / horizontal_cell_block_count as i32
    }

    fn start_cell_z(&self, horizontal_cell_block_count: u8) -> i32 {
        self.start_block_z() / horizontal_cell_block_count as i32
    }

    fn start_block_x(&self) -> i32 {
        start_block_x(&self.chunk_pos)
    }

    fn start_block_z(&self) -> i32 {
        start_block_z(&self.chunk_pos)
    }
}

#[async_trait]
impl BlockAccessor for ProtoChunk {
    async fn get_block(&self, position: &BlockPos) -> &'static pumpkin_data::Block {
        self.get_block_state(&position.0).to_block()
    }

    async fn get_block_state(&self, position: &BlockPos) -> &'static pumpkin_data::BlockState {
        self.get_block_state(&position.0).to_state()
    }

    async fn get_block_and_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Block, &'static BlockState) {
        let id = self.get_block_state(&position.0);
        BlockState::from_id_with_block(id.0)
    }
}

/// Represents the different stages of chunk generation
/// This provides type safety and ensures chunks progress through the correct stages
#[derive(Debug)]
pub enum StagedChunk {
    /// Initial empty chunk, ready for biome population
    Empty(ProtoChunk),
    /// Chunk with biomes populated, ready for noise generation
    Biomes(ProtoChunk),
    /// Chunk with terrain noise generated, ready for surface building
    Noise(ProtoChunk),
    /// Chunk with surface built, ready for features and structures
    Surface(ProtoChunk),
    /// Chunk with features and structures, ready for finalization
    Features(ProtoChunk),
    /// Fully generated chunk
    Full(ProtoChunk),
}

impl StagedChunk {
    /// Create a new empty staged chunk
    pub fn new(
        chunk_pos: Vector2<i32>,
        settings: &GenerationSettings,
        default_block: &'static BlockState,
        biome_mixer_seed: i64,
    ) -> Self {
        let proto_chunk = ProtoChunk::new(chunk_pos, settings, default_block, biome_mixer_seed);
        StagedChunk::Empty(proto_chunk)
    }

    pub fn populate_biomes(
        &mut self,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
        dimension: Dimension,
    ) {
        match self {
            StagedChunk::Empty(chunk) => {
                // Populate biomes
                let chunk_pos = chunk.chunk_pos;
                let start_x = start_block_x(&chunk_pos);
                let start_z = start_block_z(&chunk_pos);
                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(16);
                let multi_noise_config =
                    super::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(
                        biome_pos.x,
                        biome_pos.y,
                        horizontal_biome_end as usize,
                    );
                let mut multi_noise_sampler =
                    MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);

                chunk.populate_biomes(dimension, &mut multi_noise_sampler);
            }
            _ => panic!(),
        }
        unsafe {
            let data = match ptr::read(self) {
                StagedChunk::Empty(s) => s,
                _ => panic!(),
            };

            ptr::write(self, StagedChunk::Biomes(data));
        }
    }

    pub fn populate_noise(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
    ) {
        match self {
            StagedChunk::Biomes(chunk) => {
                // Generate noise
                let chunk_pos = chunk.chunk_pos;
                let generation_shape = &settings.shape;
                let horizontal_cell_count =
                    super::chunk_noise::CHUNK_DIM / generation_shape.horizontal_cell_block_count();
                let start_x = super::positions::chunk_pos::start_block_x(&chunk_pos);
                let start_z = super::positions::chunk_pos::start_block_z(&chunk_pos);

                let sampler = super::aquifer_sampler::FluidLevelSampler::Chunk(Box::new(
                    StandardChunkFluidLevelSampler::new(
                        super::aquifer_sampler::FluidLevel::new(
                            settings.sea_level,
                            settings.default_fluid.name,
                        ),
                        super::aquifer_sampler::FluidLevel::new(-54, &pumpkin_data::Block::LAVA),
                    ),
                ));

                let mut noise_sampler = super::chunk_noise::ChunkNoiseGenerator::new(
                    &noise_router.noise,
                    random_config,
                    horizontal_cell_count as usize,
                    start_x,
                    start_z,
                    generation_shape,
                    sampler,
                    settings.aquifers_enabled,
                    settings.ore_veins_enabled,
                );

                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(
                    horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
                );
                let surface_config = super::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                    biome_pos.x,
                    biome_pos.y,
                    horizontal_biome_end as usize,
                    generation_shape.min_y as i32,
                    generation_shape.max_y() as i32,
                    generation_shape.vertical_cell_block_count() as usize,
                );
                let mut surface_height_estimate_sampler = super::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
                    &noise_router.surface_estimator,
                    &surface_config,
                );

                chunk.populate_noise(&mut noise_sampler, &mut surface_height_estimate_sampler);
            }
            _ => panic!(),
        }
        unsafe {
            let data = match ptr::read(self) {
                StagedChunk::Biomes(s) => s,
                _ => panic!(),
            };

            ptr::write(self, StagedChunk::Noise(data));
        }
    }

    pub fn build_surface(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
    ) {
        match self {
            StagedChunk::Noise(chunk) => {
                // Build surface
                let chunk_pos = chunk.chunk_pos;
                let start_x = super::positions::chunk_pos::start_block_x(&chunk_pos);
                let start_z = super::positions::chunk_pos::start_block_z(&chunk_pos);
                let generation_shape = &settings.shape;
                let horizontal_cell_count =
                    super::chunk_noise::CHUNK_DIM / generation_shape.horizontal_cell_block_count();

                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(
                    horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
                );
                let surface_config = super::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                    biome_pos.x,
                    biome_pos.y,
                    horizontal_biome_end as usize,
                    generation_shape.min_y as i32,
                    generation_shape.max_y() as i32,
                    generation_shape.vertical_cell_block_count() as usize,
                );
                let mut surface_height_estimate_sampler = super::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
                    &noise_router.surface_estimator,
                    &surface_config,
                );

                chunk.build_surface(
                    settings,
                    random_config,
                    terrain_cache,
                    &mut surface_height_estimate_sampler,
                );
            }
            _ => panic!(),
        }
        unsafe {
            let data = match ptr::read(self) {
                StagedChunk::Noise(s) => s,
                _ => panic!(),
            };

            ptr::write(self, StagedChunk::Surface(data));
        }
    }

    /// Advance the chunk to the next generation stage
    pub fn advance(
        self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
        dimension: Dimension,
    ) -> Result<Self, &'static str> {
        match self {
            StagedChunk::Empty(mut chunk) => {
                // Populate biomes
                let chunk_pos = chunk.chunk_pos;
                let start_x = super::positions::chunk_pos::start_block_x(&chunk_pos);
                let start_z = super::positions::chunk_pos::start_block_z(&chunk_pos);
                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(16);
                let multi_noise_config =
                    super::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(
                        biome_pos.x,
                        biome_pos.y,
                        horizontal_biome_end as usize,
                    );
                let mut multi_noise_sampler =
                    super::noise::router::multi_noise_sampler::MultiNoiseSampler::generate(
                        &noise_router.multi_noise,
                        &multi_noise_config,
                    );

                chunk.populate_biomes(dimension, &mut multi_noise_sampler);
                Ok(StagedChunk::Biomes(chunk))
            }
            StagedChunk::Biomes(mut chunk) => {
                // Generate noise
                let chunk_pos = chunk.chunk_pos;
                let generation_shape = &settings.shape;
                let horizontal_cell_count =
                    super::chunk_noise::CHUNK_DIM / generation_shape.horizontal_cell_block_count();
                let start_x = super::positions::chunk_pos::start_block_x(&chunk_pos);
                let start_z = super::positions::chunk_pos::start_block_z(&chunk_pos);

                let sampler = super::aquifer_sampler::FluidLevelSampler::Chunk(Box::new(
                    StandardChunkFluidLevelSampler::new(
                        super::aquifer_sampler::FluidLevel::new(
                            settings.sea_level,
                            settings.default_fluid.name,
                        ),
                        super::aquifer_sampler::FluidLevel::new(-54, &pumpkin_data::Block::LAVA),
                    ),
                ));

                let mut noise_sampler = super::chunk_noise::ChunkNoiseGenerator::new(
                    &noise_router.noise,
                    random_config,
                    horizontal_cell_count as usize,
                    start_x,
                    start_z,
                    generation_shape,
                    sampler,
                    settings.aquifers_enabled,
                    settings.ore_veins_enabled,
                );

                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(
                    horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
                );
                let surface_config = super::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                    biome_pos.x,
                    biome_pos.y,
                    horizontal_biome_end as usize,
                    generation_shape.min_y as i32,
                    generation_shape.max_y() as i32,
                    generation_shape.vertical_cell_block_count() as usize,
                );
                let mut surface_height_estimate_sampler = super::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
                    &noise_router.surface_estimator,
                    &surface_config,
                );

                chunk.populate_noise(&mut noise_sampler, &mut surface_height_estimate_sampler);
                Ok(StagedChunk::Noise(chunk))
            }
            StagedChunk::Noise(mut chunk) => {
                // Build surface
                let chunk_pos = chunk.chunk_pos;
                let start_x = super::positions::chunk_pos::start_block_x(&chunk_pos);
                let start_z = super::positions::chunk_pos::start_block_z(&chunk_pos);
                let generation_shape = &settings.shape;
                let horizontal_cell_count =
                    super::chunk_noise::CHUNK_DIM / generation_shape.horizontal_cell_block_count();

                let biome_pos = Vector2::new(
                    biome_coords::from_block(start_x),
                    biome_coords::from_block(start_z),
                );
                let horizontal_biome_end = biome_coords::from_block(
                    horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
                );
                let surface_config = super::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                    biome_pos.x,
                    biome_pos.y,
                    horizontal_biome_end as usize,
                    generation_shape.min_y as i32,
                    generation_shape.max_y() as i32,
                    generation_shape.vertical_cell_block_count() as usize,
                );
                let mut surface_height_estimate_sampler = super::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
                    &noise_router.surface_estimator,
                    &surface_config,
                );

                chunk.build_surface(
                    settings,
                    random_config,
                    terrain_cache,
                    &mut surface_height_estimate_sampler,
                );
                Ok(StagedChunk::Surface(chunk))
            }
            StagedChunk::Surface(mut chunk) => {
                // Generate features and structures
                // chunk.generate_features_and_structure(level, block_registry, random_config);
                Ok(StagedChunk::Features(chunk))
            }
            StagedChunk::Features(chunk) => {
                // Mark as fully generated
                Ok(StagedChunk::Full(chunk))
            }
            StagedChunk::Full(_) => Err("Chunk is already fully generated"),
        }
    }

    /// Get the current generation stage as a string
    pub fn stage_name(&self) -> &'static str {
        match self {
            StagedChunk::Empty(_) => "Empty",
            StagedChunk::Biomes(_) => "Biomes",
            StagedChunk::Noise(_) => "Noise",
            StagedChunk::Surface(_) => "Surface",
            StagedChunk::Features(_) => "Features",
            StagedChunk::Full(_) => "Full",
        }
    }

    pub fn stage_id(&self) -> u8 {
        match self {
            StagedChunk::Empty(_) => 1,
            StagedChunk::Biomes(_) => 2,
            StagedChunk::Noise(_) => 3,
            StagedChunk::Surface(_) => 4,
            StagedChunk::Features(_) => 5,
            StagedChunk::Full(_) => 6,
        }
    }

    /// Check if the chunk is fully generated
    pub fn is_complete(&self) -> bool {
        matches!(self, StagedChunk::Full(_))
    }

    /// Get an immutable reference to the underlying ProtoChunk
    pub fn proto_chunk(&self) -> &ProtoChunk {
        match self {
            StagedChunk::Empty(chunk) => chunk,
            StagedChunk::Biomes(chunk) => chunk,
            StagedChunk::Noise(chunk) => chunk,
            StagedChunk::Surface(chunk) => chunk,
            StagedChunk::Features(chunk) => chunk,
            StagedChunk::Full(chunk) => chunk,
        }
    }

    /// Get a mutable reference to the underlying ProtoChunk
    pub fn proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            StagedChunk::Empty(chunk) => chunk,
            StagedChunk::Biomes(chunk) => chunk,
            StagedChunk::Noise(chunk) => chunk,
            StagedChunk::Surface(chunk) => chunk,
            StagedChunk::Features(chunk) => chunk,
            StagedChunk::Full(chunk) => chunk,
        }
    }

    /// Finalize the chunk, extracting the ProtoChunk if fully generated
    pub fn finalize(
        self,
        generation_settings: &GenerationSettings,
        dimension: Dimension,
    ) -> Result<ChunkData, Self> {
        match self {
            StagedChunk::Features(proto_chunk) => {
                let height: usize = match dimension {
                    Dimension::Overworld => 384,
                    Dimension::Nether | Dimension::End => 256,
                };
                let sub_chunks = height / BlockPalette::SIZE;
                let sections = (0..sub_chunks).map(|_| SubChunk::default()).collect();
                let mut sections =
                    ChunkSections::new(sections, generation_settings.shape.min_y as i32);

                for y in 0..biome_coords::from_block(generation_settings.shape.height) {
                    let relative_y = y as usize;
                    let section_index = relative_y / BiomePalette::SIZE;
                    let relative_y = relative_y % BiomePalette::SIZE;
                    if let Some(section) = sections.sections.get_mut(section_index) {
                        for z in 0..BiomePalette::SIZE {
                            for x in 0..BiomePalette::SIZE {
                                let absolute_y = biome_coords::from_block(
                                    generation_settings.shape.min_y as i32,
                                ) + y as i32;
                                let biome = proto_chunk
                                    .get_biome(&Vector3::new(x as i32, absolute_y, z as i32));
                                section.biomes.set(x, relative_y, z, biome.id);
                            }
                        }
                    }
                }
                for y in 0..generation_settings.shape.height {
                    let relative_y = (y as i32 - sections.min_y) as usize;
                    let section_index = relative_y / BlockPalette::SIZE;
                    let relative_y = relative_y % BlockPalette::SIZE;
                    if let Some(section) = sections.sections.get_mut(section_index) {
                        for z in 0..BlockPalette::SIZE {
                            for x in 0..BlockPalette::SIZE {
                                let absolute_y = generation_settings.shape.min_y as i32 + y as i32;
                                let block = proto_chunk
                                    .get_block_state(&Vector3::new(x as i32, absolute_y, z as i32));
                                section.block_states.set(x, relative_y, z, block.0);
                            }
                        }
                    }
                }
                let mut chunk = ChunkData {
                    light_engine: ChunkLight {
                        sky_light: (0..sections.sections.len() + 2)
                            .map(|_| LightContainer::new_filled(15))
                            .collect(),
                        block_light: (0..sections.sections.len() + 2)
                            .map(|_| LightContainer::new_empty(15))
                            .collect(),
                    },
                    section: sections,
                    heightmap: Default::default(),
                    position: proto_chunk.chunk_pos,
                    dirty: true,
                    block_ticks: Default::default(),
                    fluid_ticks: Default::default(),
                    block_entities: Default::default(),
                };

                chunk.heightmap = chunk.calculate_heightmap();
                Ok(chunk)
            }
            other => Err(other),
        }
    }

    /// Advance through all remaining stages to completion
    pub fn advance_to_completion(
        mut self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
        dimension: crate::dimension::Dimension,
    ) -> Result<ChunkData, &'static str> {
        while !self.is_complete() {
            self = self.advance(
                level,
                block_registry,
                settings,
                random_config,
                terrain_cache,
                noise_router,
                dimension,
            )?;
        }
        self.finalize(settings, dimension)
            .map_err(|_| "Failed to finalize chunk")
    }
}

/// Example usage of StagedChunk
impl StagedChunk {
    /// Create a StagedChunk and advance it through all stages using the same dependencies as VanillaGenerator
    pub fn generate_complete_from_vanilla_generator(
        chunk_pos: Vector2<i32>,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        noise_router: &super::noise::router::proto_noise_router::ProtoNoiseRouters,
        dimension: crate::dimension::Dimension,
        default_block: &'static BlockState,
    ) -> Result<ChunkData, &'static str> {
        // Calculate biome mixer seed
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(random_config.seed);

        // Create staged chunk and advance through all stages
        let staged_chunk = StagedChunk::new(chunk_pos, settings, default_block, biome_mixer_seed);
        staged_chunk.advance_to_completion(
            level,
            block_registry,
            settings,
            random_config,
            terrain_cache,
            noise_router,
            dimension,
        )
    }
}

#[cfg(test)]
#[allow(dead_code)] // TODO: Fix tests to work with new ProtoChunk API
mod test {
    /*
    TODO: Update all tests to work with the new ProtoChunk API that doesn't use lifetimes.
    The new API requires passing noise samplers and other dependencies as parameters to methods
    instead of storing them in the struct.

    const SEED: u64 = 0;
    static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
        LazyLock::new(|| GlobalRandomConfig::new(SEED, false)); // TODO: use legacy when needed
    static TERRAIN_CACHE: LazyLock<TerrainCache> =
        LazyLock::new(|| TerrainCache::from_random(&RANDOM_CONFIG));
    static BASE_NOISE_ROUTER: LazyLock<ProtoNoiseRouters> =
        LazyLock::new(|| ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG));

    const SEED2: u64 = 13579;
    static RANDOM_CONFIG2: LazyLock<GlobalRandomConfig> =
        LazyLock::new(|| GlobalRandomConfig::new(SEED2, false)); // TODO: use legacy when needed
    static BASE_NOISE_ROUTER2: LazyLock<ProtoNoiseRouters> = LazyLock::new(|| {
        ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG2)
    });

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_only_cell_cache() {
        // We say no wrapper, but it technically has a top-level cell cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_only_cell_2d_cache() {
        // it technically has a top-level cell cache
        // should be the same as only cell_cache
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_only_cell_cache_0_0.chunk");

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::Cache2D => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_only_cell_flat_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_flat_cache_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::CacheFlat => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_only_cell_once_cache() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_once_cache_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::CacheOnce => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_only_cell_interpolated() {
        // it technically has a top-level cell cache
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_only_cell_cache_interpolated_0_0.chunk"
        );

        let mut base_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &RANDOM_CONFIG);

        macro_rules! set_wrappers {
            ($stack: expr) => {
                $stack.iter_mut().for_each(|component| {
                    if let ProtoNoiseFunctionComponent::Wrapper(wrapper) = component {
                        match wrapper.wrapper_type {
                            WrapperType::CellCache => (),
                            WrapperType::Interpolated => (),
                            _ => {
                                *component =
                                    ProtoNoiseFunctionComponent::PassThrough(PassThrough::new(
                                        wrapper.input_index,
                                        wrapper.min(),
                                        wrapper.max(),
                                    ));
                            }
                        }
                    }
                });
            };
        }

        set_wrappers!(base_router.noise.full_component_stack);
        set_wrappers!(base_router.surface_estimator.full_component_stack);
        set_wrappers!(base_router.multi_noise.full_component_stack);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &base_router,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("{expected} vs {actual} ({index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard() {
        let _expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_0_0.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        // TODO: Create ProtoChunk and call populate_noise with proper parameters
        let _chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            surface_config,
            surface_config.default_block.get_state(),
            0, // biome_mixer_seed
        );

        // assert_eq!(
        //     expected_data,
        //     chunk.flat_block_map.into_iter().collect::<Vec<u16>>()
        // );
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_aquifer() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_7_4.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(7, 4),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        assert_eq!(
            expected_data,
            chunk.flat_block_map.into_iter().collect::<Vec<u16>>()
        );
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_badlands() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_-595_544.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-595, 544),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_frozen_ocean() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_-119_183.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-119, 183),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_badlands2() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_13579_-6_11.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-6, 11),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_badlands3() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_13579_-2_15.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-2, 15),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );
        chunk.populate_noise();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_0_0.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(0, 0),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface_badlands() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_badlands_-595_544.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-595, 544),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface_badlands2() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-6_11.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);
        let mut chunk = ProtoChunk::new(
            Vector2::new(-6, 11),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface_badlands3() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-7_9.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);

        let mut chunk = ProtoChunk::new(
            Vector2::new(-7, 9),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface_biome_blend() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_surface_13579_-2_15.chunk");
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let terrain_cache = TerrainCache::from_random(&RANDOM_CONFIG2);

        let mut chunk = ProtoChunk::new(
            Vector2::new(-2, 15),
            &BASE_NOISE_ROUTER2,
            &RANDOM_CONFIG2,
            surface_config,
            &terrain_cache,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    }

    #[test]
    #[ignore] // TODO: Update this test to work with new API
    fn test_no_blend_no_beard_surface_frozen_ocean() {
        let expected_data: Vec<u16> = read_data_from_file!(
            "../../assets/no_blend_no_beard_surface_frozen_ocean_-119_183.chunk"
        );
        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut chunk = ProtoChunk::new(
            Vector2::new(-119, 183),
            &BASE_NOISE_ROUTER,
            &RANDOM_CONFIG,
            surface_config,
            &TERRAIN_CACHE,
            surface_config.default_block.get_state(),
        );

        chunk.populate_biomes(Dimension::Overworld);
        chunk.populate_noise();
        chunk.build_surface();

        expected_data
            .into_iter()
            .zip(chunk.flat_block_map)
            .enumerate()
            .for_each(|(index, (expected, actual))| {
                if expected != actual {
                    panic!("expected {expected}, was {actual} (at {index})");
                }
            });
    */
}
