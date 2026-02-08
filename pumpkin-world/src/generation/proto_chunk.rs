use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::block_properties::is_air;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::structures::{
    Structure, StructureKeys, StructurePlacementCalculator, StructureSet, WeightedEntry,
};
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockState, block_properties::blocks_movement, chunk::Biome};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::random::{RandomImpl, get_carver_seed};
use pumpkin_util::{
    HeightMap,
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, get_decorator_seed, xoroshiro128::Xoroshiro},
};
use rustc_hash::FxHashMap;

use super::{
    GlobalRandomConfig, biome_coords, carver,
    feature::placed_features::PLACED_FEATURES,
    noise::router::{
        multi_noise_sampler::MultiNoiseSampler, proto_noise_router::DoublePerlinNoiseBuilder,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
    positions::chunk_pos::{start_block_x, start_block_z},
    section_coords,
    surface::{MaterialRuleContext, estimate_surface_height, terrain::SurfaceTerrainBuilder},
};
use crate::chunk::{ChunkData, ChunkHeightmapType};
use crate::chunk_system::StagedChunkEnum;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::noise::aquifer_sampler::{
    FluidLevel, FluidLevelSampler, FluidLevelSamplerImpl,
};
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions;
use crate::generation::noise::{CHUNK_DIM, ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK};
use crate::generation::structure::placement::should_generate_structure;
use crate::generation::structure::structures::StructureInstance;
use crate::generation::structure::try_generate_structure;
use crate::generation::surface::rule::try_apply_material_rule;
use crate::{
    BlockStateId, ProtoNoiseRouters,
    biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier},
    block::RawBlockState,
    chunk::CHUNK_AREA,
    generation::{biome, positions::chunk_pos},
    world::{BlockAccessor, BlockRegistryExt},
};

pub trait GenerationCache: HeightLimitView + BlockAccessor {
    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk;
    fn get_center_chunk(&self) -> &ProtoChunk;

    fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk>;
    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk>;

    fn try_get_proto_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk>;

    fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState;
    fn get_fluid_and_fluid_state(&self, position: &Vector3<i32>) -> (Fluid, FluidState);
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState);
    fn mark_pos_for_postprocessing(&mut self, pos: &Vector3<i32>);
    fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32;
    fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32;
    fn is_air(&self, local_pos: &Vector3<i32>) -> bool;
    fn get_biome_for_terrain_gen(&self, x: i32, y: i32, z: i32) -> &'static Biome;
}

const AIR_BLOCK: Block = Block::AIR;
type AdditionalCarvingMask = Arc<dyn Fn(i32, i32, i32) -> bool + Send + Sync>;

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

/// Vanilla Chunk Steps
///
/// 1. empty: The chunk is not yet loaded or generated.
///
/// 2. `structures_starts`: This step calculates the starting points for structure pieces. For structures that are the starting in this chunk, the position of all pieces are generated and stored.
///
/// 3. `structures_references`: A reference to nearby chunks that have a structures' starting point are stored.
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
/// 9. `initialize_light`: The lighting engine is initialized and light sources are identified.
///
/// 10. light: The lighting engine calculates the light level for blocks.
///
/// 11. spawn: Mobs are spawned.
///
/// 12. full: Generation is done and a chunk can now be loaded. The proto-chunk is now converted to a level chunk and all block updates deferred in the above steps are executed.
///
#[derive(Clone)]
pub struct ProtoChunk {
    pub x: i32,
    pub z: i32,
    pub default_block: &'static BlockState,
    biome_mixer_seed: i64,
    // These are local positions
    flat_block_map: Box<[BlockStateId]>,
    pub flat_biome_map: Box<[u8]>,
    /// HEIGHTMAPS
    ///
    /// Top block that is not air
    pub flat_surface_height_map: Box<[i16]>,
    flat_ocean_floor_height_map: Box<[i16]>,
    pub flat_motion_blocking_height_map: Box<[i16]>,
    pub flat_motion_blocking_no_leaves_height_map: Box<[i16]>,
    structure_starts: FxHashMap<StructureKeys, StructureInstance>,
    post_processing_positions: HashSet<BlockPos>,
    carving_masks: HashMap<carver::CarvingStage, carver::CarvingMask>,
    carving_mask_storage: HashMap<carver::CarvingStage, Box<[i64]>>,
    carving_mask_additional: HashMap<carver::CarvingStage, AdditionalCarvingMask>,
    pub blending_data: Option<NbtCompound>,
    old_generation_bounds: Option<OldGenerationBounds>,

    // Height of the chunk for indexing
    height: u16,
    bottom_y: i8,
    pub stage: StagedChunkEnum,
    upgrading: bool,
    old_noise_generation: bool,
}

#[derive(Clone, Copy)]
struct OldGenerationBounds {
    min_section_y: i8,
    max_section_y: i8,
}

fn old_generation_bounds_from_nbt(data: &NbtCompound) -> Option<OldGenerationBounds> {
    let min_section_y = data.get_int("min_section")?;
    let max_section_y = data.get_int("max_section")?;
    Some(OldGenerationBounds {
        min_section_y: min_section_y as i8,
        max_section_y: max_section_y as i8,
    })
}

pub struct TerrainCache {
    pub terrain_builder: SurfaceTerrainBuilder,
    pub surface_noise: DoublePerlinNoiseSampler,
    pub secondary_noise: DoublePerlinNoiseSampler,
}

struct NoiseCellContext<'a, 'b> {
    noise_sampler: &'a mut ChunkNoiseGenerator<'b>,
    random_config: &'a GlobalRandomConfig,
    surface_height_estimate_sampler: &'a mut SurfaceHeightEstimateSampler<'b>,
    h_count: i32,
    v_count: i32,
    cell_height: u16,
    minimum_cell_y: i8,
    delta_y_step: f64,
    delta_x_z_step: f64,
}

struct NoiseCellOrigin {
    sample_start_x: i32,
    sample_start_z: i32,
    block_x_base: i32,
    block_z_base: i32,
    cell_x: i32,
    cell_z: i32,
}

impl TerrainCache {
    #[must_use]
    pub fn from_random(random_config: &GlobalRandomConfig) -> Self {
        let random = &random_config.base_random_deriver;
        let noise_builder = DoublePerlinNoiseBuilder::new(random_config);
        let terrain_builder = SurfaceTerrainBuilder::new(&noise_builder, random);
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
    #[must_use]
    pub fn new(
        x: i32,
        z: i32,
        dimension: &Dimension,
        default_block: &'static BlockState,
        biome_mixer_seed: i64,
    ) -> Self {
        let height = dimension.logical_height as u16;

        let default_heightmap = vec![i16::MIN; CHUNK_AREA].into_boxed_slice();
        Self {
            x,
            z,
            default_block,
            flat_block_map: vec![0; CHUNK_AREA * height as usize].into_boxed_slice(),
            flat_biome_map: vec![
                Biome::PLAINS.id;
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
            structure_starts: FxHashMap::default(),
            post_processing_positions: HashSet::new(),
            carving_masks: HashMap::new(),
            carving_mask_storage: HashMap::new(),
            carving_mask_additional: HashMap::new(),
            blending_data: None,
            old_generation_bounds: None,
            height,
            bottom_y: dimension.min_y as i8,
            stage: StagedChunkEnum::Empty,
            upgrading: false,
            old_noise_generation: false,
        }
    }

    pub async fn from_chunk_data(
        chunk_data: &ChunkData,
        dimension: &Dimension,
        default_block: &'static BlockState,
        biome_mixer_seed: i64,
    ) -> Self {
        let mut proto_chunk = Self::new(
            chunk_data.x,
            chunk_data.z,
            dimension,
            default_block,
            biome_mixer_seed,
        );

        proto_chunk.old_noise_generation = chunk_data
            .blending_data
            .as_ref()
            .and_then(|data| {
                data.get_bool("old_noise_generation")
                    .or_else(|| data.get_bool("old_noise"))
            })
            .unwrap_or(false);
        proto_chunk.old_generation_bounds = chunk_data
            .blending_data
            .as_ref()
            .and_then(old_generation_bounds_from_nbt);
        proto_chunk.blending_data = chunk_data.blending_data.clone();

        proto_chunk.set_carving_mask_data(
            carver::CarvingStage::Air,
            chunk_data.carving_mask_air.clone(),
        );
        proto_chunk.set_carving_mask_data(
            carver::CarvingStage::Liquid,
            chunk_data.carving_mask_liquid.clone(),
        );
        if chunk_data.carving_mask_liquid.is_empty() {
            proto_chunk.set_carving_mask_data(
                carver::CarvingStage::Liquid,
                chunk_data.carving_mask_air.clone(),
            );
        }

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

        proto_chunk
    }

    pub fn set_additional_carving_mask(
        &mut self,
        stage: carver::CarvingStage,
        mask: AdditionalCarvingMask,
    ) {
        self.carving_mask_additional
            .insert(stage, Arc::clone(&mask));
        if let Some(existing) = self.carving_masks.get_mut(&stage) {
            let mask = Arc::clone(&mask);
            existing
                .set_additional_mask(move |offset_x, y, offset_z| (mask)(offset_x, y, offset_z));
        }
    }

    fn apply_additional_mask(&self, stage: carver::CarvingStage, mask: &mut carver::CarvingMask) {
        let Some(additional) = self.carving_mask_additional.get(&stage) else {
            return;
        };
        let additional = Arc::clone(additional);
        mask.set_additional_mask(move |offset_x, y, offset_z| (additional)(offset_x, y, offset_z));
    }

    pub fn take_carving_mask(&mut self, stage: carver::CarvingStage) -> carver::CarvingMask {
        let mut mask = if let Some(mask_data) = self.carving_mask_storage.remove(&stage) {
            carver::CarvingMask::from_long_array(self.height, self.bottom_y, &mask_data)
        } else {
            self.carving_masks
                .remove(&stage)
                .unwrap_or_else(|| carver::CarvingMask::new(self.height, self.bottom_y))
        };
        self.apply_additional_mask(stage, &mut mask);
        mask
    }

    pub fn get_or_create_carving_mask(
        &mut self,
        stage: carver::CarvingStage,
    ) -> &mut carver::CarvingMask {
        if let Some(mask_data) = self.carving_mask_storage.remove(&stage) {
            let mut mask =
                carver::CarvingMask::from_long_array(self.height, self.bottom_y, &mask_data);
            self.apply_additional_mask(stage, &mut mask);
            self.carving_masks.insert(stage, mask);
        }
        if !self.carving_masks.contains_key(&stage) {
            let mut mask = carver::CarvingMask::new(self.height, self.bottom_y);
            self.apply_additional_mask(stage, &mut mask);
            self.carving_masks.insert(stage, mask);
        }
        let additional = self.carving_mask_additional.get(&stage).cloned();
        let mask = self
            .carving_masks
            .get_mut(&stage)
            .expect("carving mask exists");
        if let Some(additional) = additional {
            let additional = Arc::clone(&additional);
            mask.set_additional_mask(move |offset_x, y, offset_z| {
                (additional)(offset_x, y, offset_z)
            });
        }
        mask
    }

    pub fn store_carving_mask(&mut self, stage: carver::CarvingStage, mask: carver::CarvingMask) {
        self.carving_masks.insert(stage, mask);
        self.carving_mask_storage.remove(&stage);
    }

    #[must_use]
    pub fn carving_mask_data(&self, stage: carver::CarvingStage) -> Option<Box<[i64]>> {
        if let Some(mask) = self.carving_masks.get(&stage) {
            return Some(mask.to_long_array());
        }
        self.carving_mask_storage.get(&stage).cloned()
    }

    pub fn set_carving_mask_data(&mut self, stage: carver::CarvingStage, data: Box<[i64]>) {
        self.carving_mask_storage.insert(stage, data);
        self.carving_masks.remove(&stage);
    }

    pub fn mark_pos_for_postprocessing(&mut self, pos: BlockPos) {
        self.post_processing_positions.insert(pos);
    }

    pub fn drain_postprocessing_positions(&mut self) -> Vec<BlockPos> {
        self.post_processing_positions.drain().collect()
    }

    #[must_use]
    pub fn post_processing_positions(&self) -> &HashSet<BlockPos> {
        &self.post_processing_positions
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
    pub const fn is_upgrading(&self) -> bool {
        self.upgrading
    }

    #[must_use]
    pub const fn is_old_noise_generation(&self) -> bool {
        self.old_noise_generation
    }

    #[must_use]
    pub fn has_blending_data(&self) -> bool {
        self.blending_data
            .as_ref()
            .is_some_and(|data| !data.is_empty())
    }

    #[must_use]
    pub fn blending_data_old_generation_bounds(&self) -> Option<(i32, i32)> {
        self.old_generation_bounds.map(|bounds| {
            let min_y = section_coords::section_to_block(bounds.min_section_y as i32);
            let max_section = bounds.max_section_y as i32;
            let max_y_exclusive = section_coords::section_to_block(max_section);
            (min_y, max_y_exclusive - min_y)
        })
    }

    pub fn set_upgrading(&mut self, upgrading: bool) {
        self.upgrading = upgrading;
    }

    pub fn set_old_noise_generation(&mut self, old_noise_generation: bool) {
        self.old_noise_generation = old_noise_generation;
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

    fn recompute_heightmaps_for_column(&mut self, local_x: i32, local_z: i32) {
        let mut surface: i16 = i16::MIN;
        let mut ocean_floor: i16 = i16::MIN;
        let mut motion_blocking: i16 = i16::MIN;
        let mut motion_blocking_no_leaves: i16 = i16::MIN;
        let min_y = self.bottom_y() as i32;
        let max_y = min_y + self.height() as i32 - 1;

        for y in (min_y..=max_y).rev() {
            let state_id = self.get_block_state_raw(local_x, y - min_y, local_z);
            if is_air(state_id) {
                continue;
            }

            if surface == i16::MIN {
                surface = y as i16;
            }

            let block = Block::get_raw_id_from_state_id(state_id);
            let state = BlockState::from_id(state_id);
            let blocks_movement = blocks_movement(state, block);
            if blocks_movement && ocean_floor == i16::MIN {
                ocean_floor = y as i16;
            }
            let is_motion_blocking = blocks_movement || state.is_liquid();
            if is_motion_blocking {
                if motion_blocking == i16::MIN {
                    motion_blocking = y as i16;
                }
                if motion_blocking_no_leaves == i16::MIN
                    && !tag::Block::MINECRAFT_LEAVES.1.contains(&block)
                {
                    motion_blocking_no_leaves = y as i16;
                }
            }

            if surface != i16::MIN
                && ocean_floor != i16::MIN
                && motion_blocking != i16::MIN
                && motion_blocking_no_leaves != i16::MIN
            {
                break;
            }
        }

        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_surface_height_map[index] = surface;
        self.flat_ocean_floor_height_map[index] = ocean_floor;
        self.flat_motion_blocking_height_map[index] = motion_blocking;
        self.flat_motion_blocking_no_leaves_height_map[index] = motion_blocking_no_leaves;
    }

    #[must_use]
    pub fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32 {
        match heightmap {
            HeightMap::WorldSurfaceWg => self.top_block_height_exclusive(x, z),
            HeightMap::WorldSurface => self.top_block_height_exclusive(x, z),
            HeightMap::OceanFloorWg => self.ocean_floor_height_exclusive(x, z),
            HeightMap::OceanFloor => self.ocean_floor_height_exclusive(x, z),
            HeightMap::MotionBlocking => self.top_motion_blocking_block_height_exclusive(x, z),
            HeightMap::MotionBlockingNoLeaves => {
                self.top_motion_blocking_block_no_leaves_height_exclusive(x, z)
            }
        }
    }

    #[must_use]
    pub fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let local_x = x & 15;
        let local_z = z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_surface_height_map[index] as i32 + 1
    }

    #[must_use]
    pub fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let local_x = x & 15;
        let local_z = z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_ocean_floor_height_map[index] as i32 + 1
    }

    #[must_use]
    pub fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let local_x = x & 15;
        let local_z = z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_motion_blocking_height_map[index] as i32 + 1
    }

    #[must_use]
    pub fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let local_x = x & 15;
        let local_z = z & 15;
        let index = Self::local_position_to_height_map_index(local_x, local_z);
        self.flat_motion_blocking_no_leaves_height_map[index] as i32 + 1
    }

    #[inline]
    const fn local_position_to_height_map_index(x: i32, z: i32) -> usize {
        x as usize * CHUNK_DIM as usize + z as usize
    }

    #[inline]
    fn local_pos_to_block_index(&self, x: i32, y: i32, z: i32) -> usize {
        debug_assert!((0..16).contains(&x), "x out of bounds: {}", x);
        debug_assert!((0..16).contains(&z), "z out of bounds: {}", z);
        debug_assert!(y >= 0 && y < self.height() as i32, "y out of bounds: {}", y);

        self.height() as usize * CHUNK_DIM as usize * x as usize
            + CHUNK_DIM as usize * y as usize
            + z as usize
    }

    #[inline]
    #[must_use]
    pub fn local_biome_pos_to_biome_index(&self, x: i32, y: i32, z: i32) -> usize {
        let biome_height = self.height() as usize >> 2;

        debug_assert!((0..4).contains(&x), "Biome X out of bounds: {}", x);
        debug_assert!((0..4).contains(&z), "Biome Z out of bounds: {}", z);
        debug_assert!(
            y >= 0 && y < biome_height as i32,
            "Biome Y out of bounds: {}",
            y
        );

        biome_height * biome_coords::from_block(CHUNK_DIM as usize) * x as usize
            + biome_coords::from_block(CHUNK_DIM as usize) * y as usize
            + z as usize
    }

    #[inline]
    #[must_use]
    pub fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        is_air(self.get_block_state(local_pos).0)
    }

    #[inline]
    #[must_use]
    pub fn get_block_state_raw(&self, x: i32, y: i32, z: i32) -> u16 {
        let index = self.local_pos_to_block_index(x, y, z);
        self.flat_block_map[index]
    }

    #[inline]
    #[must_use]
    pub fn get_block_state(&self, local_pos: &Vector3<i32>) -> RawBlockState {
        let local_y = local_pos.y - self.bottom_y() as i32;
        if local_y < 0 || local_y >= self.height() as i32 {
            return RawBlockState(Block::VOID_AIR.default_state.id);
        }
        RawBlockState(self.get_block_state_raw(local_pos.x & 15, local_y, local_pos.z & 15))
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
            let block = Block::get_raw_id_from_state_id(block_state.id);

            let blocks_movement = blocks_movement(block_state, block);
            if blocks_movement {
                self.maybe_update_ocean_floor_height_map(index, y);
            }
            if blocks_movement || block_state.is_liquid() {
                self.maybe_update_motion_blocking_height_map(index, y);
                if !tag::Block::MINECRAFT_LEAVES.1.contains(&block) {
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

    pub fn step_to_biomes(&mut self, dimension: Dimension, noise_router: &ProtoNoiseRouters) {
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);
        let horizontal_biome_end = biome_coords::from_block(16);
        let multi_noise_config =
            super::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end as usize,
            );
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);
        self.populate_biomes(dimension, &mut multi_noise_sampler);
        self.stage = StagedChunkEnum::Biomes;
    }

    pub fn step_to_noise(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        noise_router: &ProtoNoiseRouters,
    ) {
        let generation_shape = &settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);

        let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                settings.sea_level,
                Block::from_registry_key(settings.default_fluid.name).unwrap(),
            ),
            FluidLevel::new(-54, &Block::LAVA),
        ));

        let mut noise_sampler = ChunkNoiseGenerator::new(
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

        let mut surface_height_estimate_sampler = self.build_surface_height_estimate_sampler(
            settings,
            noise_router,
            start_x,
            start_z,
            horizontal_cell_count.into(),
        );
        self.populate_noise(
            &mut noise_sampler,
            random_config,
            &mut surface_height_estimate_sampler,
        );

        self.stage = StagedChunkEnum::Noise;
    }

    pub fn step_to_surface(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        noise_router: &ProtoNoiseRouters,
    ) {
        debug_assert_eq!(self.stage, StagedChunkEnum::Noise);
        // Build surface
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);
        let generation_shape = &settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();

        let mut surface_height_estimate_sampler = self.build_surface_height_estimate_sampler(
            settings,
            noise_router,
            start_x,
            start_z,
            horizontal_cell_count.into(),
        );

        self.build_surface(
            settings,
            random_config,
            terrain_cache,
            &mut surface_height_estimate_sampler,
        );
        self.stage = StagedChunkEnum::Surface;
    }

    pub fn step_to_carvers<T: GenerationCache>(
        cache: &mut T,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        noise_router: &ProtoNoiseRouters,
        dimension: Dimension,
    ) {
        crate::generation::blender::apply_carving_mask_filter(cache);
        let (chunk_x, chunk_z) = {
            let chunk = cache.get_center_chunk();
            (chunk.x, chunk.z)
        };

        let carved_columns = if dimension == Dimension::OVERWORLD {
            carver::carve_overworld(
                cache,
                settings,
                chunk_x,
                chunk_z,
                random_config,
                noise_router,
            )
        } else if dimension == Dimension::THE_NETHER {
            carver::carve_nether(
                cache,
                settings,
                chunk_x,
                chunk_z,
                random_config,
                noise_router,
            )
        } else {
            Vec::new()
        };
        if !carved_columns.is_empty() {
            let chunk = cache.get_center_chunk_mut();
            for (local_x, local_z) in carved_columns {
                chunk.recompute_heightmaps_for_column(local_x, local_z);
            }
        }
        cache.get_center_chunk_mut().stage = StagedChunkEnum::Carvers;
    }

    pub fn populate_biomes(
        &mut self,
        dimension: Dimension,
        multi_noise_sampler: &mut MultiNoiseSampler,
    ) {
        let min_y = self.bottom_y();
        let bottom_section = section_coords::block_to_section(min_y) as i32;
        let top_section = section_coords::block_to_section(min_y as i32 + self.height() as i32 - 1);

        let start_block_x = start_block_x(self.x);
        let start_block_z = start_block_z(self.z);

        let start_biome_x = biome_coords::from_block(start_block_x);
        let start_biome_z = biome_coords::from_block(start_block_z);

        for i in bottom_section..=top_section {
            let start_block_y = section_coords::section_to_block(i);
            let start_biome_y = biome_coords::from_block(start_block_y);

            let biomes_per_section = biome_coords::from_block(CHUNK_DIM) as i32;
            for x in 0..biomes_per_section {
                for y in 0..biomes_per_section {
                    for z in 0..biomes_per_section {
                        let biome = if dimension == Dimension::THE_END {
                            TheEndBiomeSupplier::biome(
                                start_biome_x + x,
                                start_biome_y + y,
                                start_biome_z + z,
                                multi_noise_sampler,
                                dimension,
                            )
                        } else {
                            MultiNoiseBiomeSupplier::biome(
                                start_biome_x + x,
                                start_biome_y + y,
                                start_biome_z + z,
                                multi_noise_sampler,
                                dimension,
                            )
                        };
                        let index = self.local_biome_pos_to_biome_index(
                            x,
                            start_biome_y + y - biome_coords::from_block(min_y as i32),
                            z,
                        );

                        self.flat_biome_map[index] = biome.id;
                    }
                }
            }
        }
    }

    #[expect(clippy::similar_names)]
    pub fn populate_noise<'a, 'b>(
        &mut self,
        noise_sampler: &'a mut ChunkNoiseGenerator<'b>,
        random_config: &'a GlobalRandomConfig,
        surface_height_estimate_sampler: &'a mut SurfaceHeightEstimateSampler<'b>,
    ) {
        let h_count = noise_sampler.horizontal_cell_block_count() as i32;
        let v_count = noise_sampler.vertical_cell_block_count() as i32;
        let horizontal_cells = CHUNK_DIM as i32 / h_count;

        let min_y = self.bottom_y();
        let minimum_cell_y = min_y / v_count as i8;
        let cell_height = self.height() / v_count as u16;

        let delta_y_step = 1.0 / v_count as f64;
        let delta_x_z_step = 1.0 / h_count as f64;

        let mut cell_context = NoiseCellContext {
            noise_sampler,
            random_config,
            surface_height_estimate_sampler,
            h_count,
            v_count,
            cell_height,
            minimum_cell_y,
            delta_y_step,
            delta_x_z_step,
        };
        cell_context.noise_sampler.sample_start_density();
        for cell_x in 0..horizontal_cells {
            cell_context.noise_sampler.sample_end_density(cell_x);
            let (sample_start_x, block_x_base) = self.cell_x_bases(h_count, cell_x);

            for cell_z in 0..horizontal_cells {
                let (sample_start_z, block_z_base) = self.cell_z_bases(h_count, cell_z);
                let origin = NoiseCellOrigin {
                    sample_start_x,
                    sample_start_z,
                    block_x_base,
                    block_z_base,
                    cell_x,
                    cell_z,
                };
                self.populate_noise_cell(&mut cell_context, origin);
            }
            cell_context.noise_sampler.swap_buffers();
        }
    }

    fn cell_x_bases(&self, horizontal_cell_block_count: i32, cell_x: i32) -> (i32, i32) {
        let sample_start_x =
            (self.start_cell_x(horizontal_cell_block_count) + cell_x) * horizontal_cell_block_count;
        let block_x_base = self.start_block_x() + cell_x * horizontal_cell_block_count;
        (sample_start_x, block_x_base)
    }

    fn cell_z_bases(&self, horizontal_cell_block_count: i32, cell_z: i32) -> (i32, i32) {
        let sample_start_z =
            (self.start_cell_z(horizontal_cell_block_count) + cell_z) * horizontal_cell_block_count;
        let block_z_base = self.start_block_z() + cell_z * horizontal_cell_block_count;
        (sample_start_z, block_z_base)
    }

    #[inline]
    fn cell_offsets(
        local_x: i32,
        block_y: i32,
        sample_start_y: i32,
        local_z: i32,
    ) -> (i32, i32, i32) {
        (local_x, block_y - sample_start_y, local_z)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_block_state_for_cell<'b>(
        &self,
        noise_sampler: &mut ChunkNoiseGenerator<'b>,
        random_config: &GlobalRandomConfig,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler<'b>,
        sample_start_x: i32,
        sample_start_y: i32,
        sample_start_z: i32,
        cell_offset_x: i32,
        cell_offset_y: i32,
        cell_offset_z: i32,
    ) -> &'static BlockState {
        noise_sampler
            .sample_block_state(
                random_config,
                sample_start_x,
                sample_start_y,
                sample_start_z,
                cell_offset_x,
                cell_offset_y,
                cell_offset_z,
                surface_height_estimate_sampler,
            )
            .unwrap_or(self.default_block)
    }

    fn populate_noise_cell(
        &mut self,
        context: &mut NoiseCellContext<'_, '_>,
        origin: NoiseCellOrigin,
    ) {
        for cell_y in (0..context.cell_height).rev() {
            context.noise_sampler.on_sampled_cell_corners(
                origin.cell_x,
                cell_y as i32,
                origin.cell_z,
            );
            let sample_start_y = (context.minimum_cell_y as i32 + cell_y as i32) * context.v_count;

            for local_y in (0..context.v_count).rev() {
                let block_y = sample_start_y + local_y;
                context
                    .noise_sampler
                    .interpolate_y(local_y as f64 * context.delta_y_step);

                for local_x in 0..context.h_count {
                    context
                        .noise_sampler
                        .interpolate_x(local_x as f64 * context.delta_x_z_step);
                    let block_x = origin.block_x_base + local_x;

                    for local_z in 0..context.h_count {
                        context
                            .noise_sampler
                            .interpolate_z(local_z as f64 * context.delta_x_z_step);
                        let block_z = origin.block_z_base + local_z;

                        let (cell_offset_x, cell_offset_y, cell_offset_z) =
                            Self::cell_offsets(local_x, block_y, sample_start_y, local_z);
                        let block_state = self.sample_block_state_for_cell(
                            context.noise_sampler,
                            context.random_config,
                            context.surface_height_estimate_sampler,
                            origin.sample_start_x,
                            sample_start_y,
                            origin.sample_start_z,
                            cell_offset_x,
                            cell_offset_y,
                            cell_offset_z,
                        );
                        self.set_block_state(block_x, block_y, block_z, block_state);
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn get_terrain_gen_biome_id(&self, x: i32, y: i32, z: i32) -> u8 {
        // TODO: See if we can cache this value
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
        Biome::from_id(self.get_terrain_gen_biome_id(x, y, z)).unwrap()
    }

    /// Applies surface rules and large surface features after biomes are assigned.
    pub fn build_surface(
        &mut self,
        settings: &GenerationSettings,
        random_config: &GlobalRandomConfig,
        terrain_cache: &TerrainCache,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let start_x = chunk_pos::start_block_x(self.x);
        let start_z = chunk_pos::start_block_z(self.z);
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

                let mut top_block = self.top_block_height_exclusive(local_x, local_z);

                let biome_y = if settings.legacy_random_source {
                    0
                } else {
                    top_block
                };

                let this_biome = self.get_terrain_gen_biome_id(x, biome_y, z);
                if this_biome == Biome::ERODED_BADLANDS {
                    terrain_cache
                        .terrain_builder
                        .place_badlands_pillar(self, x, z, top_block);

                    // Get the top block again if we placed a pillar!
                    top_block = self.top_block_height_exclusive(local_x, local_z);
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

                        for search_y in ((min_y as i32 - 1)..y).rev() {
                            if search_y < min_y as i32 {
                                min = search_y + 1;
                                break;
                            }

                            let state = self
                                .get_block_state(&Vector3::new(local_x, search_y, local_z))
                                .to_block_id();

                            // TODO: Is there a better way to check that its not a fluid?
                            if !(state != AIR_BLOCK && state != WATER_BLOCK && state != LAVA_BLOCK)
                            {
                                min = search_y + 1;
                                break;
                            }
                        }
                    }

                    stone_depth_above += 1;
                    let stone_depth_below = y - min + 1;
                    context.init_vertical(stone_depth_above, stone_depth_below, y, fluid_height);

                    if state.id == self.default_block.id {
                        context.biome = self.get_terrain_gen_biome(
                            context.block_pos_x,
                            context.block_pos_y,
                            context.block_pos_z,
                        );
                        let new_state = try_apply_material_rule(
                            &settings.surface_rule,
                            self,
                            &mut context,
                            surface_height_estimate_sampler,
                        );

                        if let Some(state) = new_state {
                            self.set_block_state(x, y, z, state);
                        }
                    }
                }
                if this_biome == Biome::FROZEN_OCEAN || this_biome == Biome::DEEP_FROZEN_OCEAN {
                    let surface_estimate =
                        estimate_surface_height(&mut context, surface_height_estimate_sampler);

                    terrain_cache.terrain_builder.place_iceberg(
                        self,
                        Biome::from_id(this_biome).unwrap(),
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

    /// Generates structure pieces and features after biomes/surface are ready.
    /// Biome feature lists drive what runs per step.
    pub fn generate_features_and_structure<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn BlockRegistryExt,
        random_config: &GlobalRandomConfig,
    ) {
        let (center_x, center_z, min_y, height, biomes_in_chunk) = {
            let chunk = cache.get_center_chunk();
            let mut unique_biomes = Vec::with_capacity(4);
            let mut seen_biomes = [false; u8::MAX as usize + 1];
            for &biome_id in &chunk.flat_biome_map {
                let index = biome_id as usize;
                if !seen_biomes[index] {
                    seen_biomes[index] = true;
                    unique_biomes.push(biome_id);
                }
            }
            (
                chunk.x,
                chunk.z,
                chunk.bottom_y() as i32,
                chunk.height() as i32,
                unique_biomes,
            )
        };

        let start_block_x = chunk_pos::start_block_x(center_x);
        let start_block_z = chunk_pos::start_block_z(center_z);
        let origin_pos = BlockPos::new(start_block_x, min_y, start_block_z);

        let population_seed =
            Xoroshiro::get_population_seed(random_config.seed, start_block_x, start_block_z);

        for step in 0..11 {
            Self::generate_structure_step(cache, step, population_seed, random_config.seed as i64);

            let mut features_to_run = Vec::new();
            for biome_id in &biomes_in_chunk {
                if let Some(biome) = Biome::from_id(*biome_id)
                    && let Some(features_at_step) = biome.features.get(step)
                {
                    for &feature_id in *features_at_step {
                        features_to_run
                            .push(feature_id.strip_prefix("minecraft:").unwrap_or(feature_id));
                    }
                }
            }

            features_to_run.sort_unstable();
            features_to_run.dedup();

            for (p, feature_id) in features_to_run.into_iter().enumerate() {
                if let Some(feature) = PLACED_FEATURES.get(feature_id) {
                    let decorator_seed = get_decorator_seed(population_seed, p as u64, step as u64);
                    let mut random =
                        RandomGenerator::Xoroshiro(Xoroshiro::from_seed(decorator_seed));

                    feature.generate(
                        cache,
                        block_registry,
                        min_y as i8,
                        height as u16,
                        feature_id,
                        &mut random,
                        origin_pos,
                    );
                }
            }
        }

        cache.get_center_chunk_mut().stage = StagedChunkEnum::Features;
    }

    fn generate_structure_step<T: GenerationCache>(
        cache: &mut T,
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
                    StructureInstance::Reference(origin_block_pos) => {
                        let origin_chunk_x = origin_block_pos.0.x >> 4;
                        let origin_chunk_z = origin_block_pos.0.z >> 4;
                        if let Some(neighbor) = cache.get_chunk(origin_chunk_x, origin_chunk_z)
                            && let Some(StructureInstance::Start(pos)) =
                                neighbor.structure_starts.get(id)
                        {
                            tasks.push(pos.collector.clone());
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
                                StructureInstance::Reference(origin_block_pos) => {
                                    let origin_chunk_x = origin_block_pos.0.x >> 4;
                                    let origin_chunk_z = origin_block_pos.0.z >> 4;
                                    if let Some(origin_neighbor) =
                                        cache.try_get_proto_chunk(origin_chunk_x, origin_chunk_z)
                                        && let Some(StructureInstance::Start(pos)) =
                                            origin_neighbor.structure_starts.get(id)
                                    {
                                        let start_x = chunk_pos::start_block_x(center_x);
                                        let start_z = chunk_pos::start_block_z(center_z);
                                        let end_x = start_x + 15;
                                        let end_z = start_z + 15;

                                        if pos
                                            .get_bounding_box()
                                            .intersects_raw_xz(start_x, start_z, end_x, end_z)
                                        {
                                            let collector_arc = pos.collector.clone();
                                            if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc))
                                            {
                                                tasks.push(collector_arc);
                                            }
                                        }
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
            let mut collector = collector_arc.lock().unwrap();
            collector.generate_in_chunk(chunk, &mut random, world_seed);
        }
    }

    pub fn set_structure_starts(
        &mut self,
        random_config: &GlobalRandomConfig,
        settings: &GenerationSettings,
    ) {
        let seed = random_config.seed;
        let calculator = StructurePlacementCalculator::new(seed as i64);

        for set in StructureSet::ALL {
            if !should_generate_structure(&set.placement, &calculator, self.x, self.z) {
                continue;
            }

            if set.structures.len() == 1 {
                if let Some(entry) = set.structures.first() {
                    self.try_set_structure_start(settings.sea_level, entry, random_config);
                }
                continue;
            }

            let mut candidates = set.structures.to_vec();
            let mut random: RandomGenerator =
                RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
            let carver_seed = get_carver_seed(&mut random, seed, self.x, self.z);
            let mut random: RandomGenerator =
                RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed));

            let mut total_weight: u32 = candidates.iter().map(|e| e.weight).sum();

            while !candidates.is_empty() {
                let mut roll = random.next_bounded_i32(total_weight as i32);
                let mut selected_idx = 0;

                for (i, entry) in candidates.iter().enumerate() {
                    roll -= entry.weight as i32;
                    if roll < 0 {
                        selected_idx = i;
                        break;
                    }
                }

                let selected_entry = &candidates[selected_idx];

                if self.try_set_structure_start(settings.sea_level, selected_entry, random_config) {
                    break;
                }

                let failed_entry = candidates.remove(selected_idx);
                total_weight -= failed_entry.weight;
            }
        }
        self.stage = StagedChunkEnum::StructureStart;
    }

    fn try_set_structure_start(
        &mut self,
        sea_level: i32,
        entry: &WeightedEntry,
        random_config: &GlobalRandomConfig,
    ) -> bool {
        let structure = Structure::get(&entry.structure);
        let position = try_generate_structure(
            &entry.structure,
            structure,
            random_config.seed as i64,
            self,
            sea_level,
        );

        if let Some(pos) = position {
            self.structure_starts
                .insert(entry.structure, StructureInstance::Start(pos));
            return true;
        }
        false
    }

    pub fn set_structure_references<T: GenerationCache>(cache: &mut T) {
        let (center_x, center_z, start_x, start_z) = {
            let chunk = cache.get_center_chunk();
            (
                chunk.x,
                chunk.z,
                chunk_pos::start_block_x(chunk.x),
                chunk_pos::start_block_z(chunk.z),
            )
        };

        let end_x = start_x + 15;
        let end_z = start_z + 15;
        let radius = 8;

        let mut references = Vec::new();

        for x in (center_x - radius)..=(center_x + radius) {
            for z in (center_z - radius)..=(center_z + radius) {
                if x == center_x && z == center_z {
                    continue;
                }

                if let Some(neighbor) = cache.get_chunk(x, z) {
                    for (key, instance) in &neighbor.structure_starts {
                        if let StructureInstance::Start(start_data) = instance
                            && start_data
                                .get_bounding_box()
                                .intersects_raw_xz(start_x, start_z, end_x, end_z)
                        {
                            references.push((*key, start_data.start_pos));
                        }
                    }
                }
            }
        }

        let center_chunk = cache.get_center_chunk_mut();
        for (key, pos) in references {
            center_chunk
                .structure_starts
                .entry(key)
                .or_insert_with(|| StructureInstance::Reference(pos));
        }

        center_chunk.stage = StagedChunkEnum::StructureReferences;
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

    fn build_surface_height_estimate_sampler<'a>(
        &self,
        settings: &GenerationSettings,
        noise_router: &'a ProtoNoiseRouters,
        start_x: i32,
        start_z: i32,
        horizontal_cell_count: i32,
    ) -> SurfaceHeightEstimateSampler<'a> {
        let generation_shape = &settings.shape;
        let horizontal_cell_block_count = i32::from(generation_shape.horizontal_cell_block_count());
        let horizontal_biome_end =
            biome_coords::from_block(horizontal_cell_count * horizontal_cell_block_count);
        let surface_config = SurfaceHeightSamplerBuilderOptions::new(
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
            generation_shape.min_y as i32,
            generation_shape.max_y() as i32,
            generation_shape.vertical_cell_block_count() as usize,
        );
        SurfaceHeightEstimateSampler::generate(&noise_router.surface_estimator, &surface_config)
    }
}

impl BlockAccessor for ProtoChunk {
    fn get_block<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static Block> + Send + 'a>> {
        Box::pin(async move { self.get_block_state(&position.0).to_block() })
    }

    fn get_block_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static BlockState> + Send + 'a>> {
        Box::pin(async move { self.get_block_state(&position.0).to_state() })
    }

    fn get_block_state_id<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = BlockStateId> + Send + 'a>> {
        Box::pin(async move { self.get_block_state(&position.0).0 })
    }

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>> {
        Box::pin(async move {
            let id = self.get_block_state(&position.0);
            BlockState::from_id_with_block(id.0)
        })
    }
}
