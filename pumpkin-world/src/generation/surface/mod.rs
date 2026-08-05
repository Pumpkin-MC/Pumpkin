use pumpkin_data::{
    chunk::Biome,
    chunk_gen_settings::{
        AboveYMaterialCondition, MaterialCondition, NoiseThresholdMaterialCondition,
        NotMaterialCondition, StoneDepthMaterialCondition, VerticalGradientMaterialCondition,
        WaterMaterialCondition,
    },
};
use pumpkin_util::{
    math::{lerp2, vertical_surface_type::VerticalSurfaceType},
    random::{RandomImpl, xoroshiro128::XoroshiroSplitter},
};

use terrain::SurfaceTerrainBuilder;

use crate::{
    ProtoChunk,
    generation::{positions::chunk_pos, section_coords},
};

use super::{
    noise::perlin::DoublePerlinNoiseSampler,
    noise::router::{
        proto_noise_router::DoublePerlinNoiseBuilder,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
};

// ---------------------------------------------------------------------------
// GPU acceleration hook — surface noise pre-computation
// ---------------------------------------------------------------------------
//
// pumpkin-world does NOT depend on pumpkin-gpu. Instead it exposes a
// function-pointer slot that the main server crate (pumpkin) can fill at
// startup. When a GPU callback is registered, the surface stage pre-computes
// per-column noise values in batch instead of calling CPU samplers.
use std::sync::OnceLock;

/// Pre-computed noise values for surface/material rule evaluation.
///
/// `surface_noise` values correspond to `surface_noise.sample(x, 0, z)` for each
/// column and are used for `run_depth` computation. `secondary_noise` values
/// correspond to `secondary_noise.sample(x, 0, z)` for `secondary_depth`.
pub struct SurfaceNoiseBatch {
    /// `surface_noise.sample(x, 0, z)` per column (256 entries, row-major over 16×16).
    pub surface_noise: std::sync::Arc<[f64]>,
    /// `secondary_noise.sample(x, 0, z)` per column (256 entries, row-major over 16×16).
    pub secondary_noise: std::sync::Arc<[f64]>,
}

/// Signature for GPU-accelerated surface noise pre-computation.
///
/// Arguments:
/// - `surface_sampler`: the `DoublePerlinNoiseSampler` used for `run_depth`
/// - `secondary_sampler`: the `DoublePerlinNoiseSampler` used for `secondary_depth`
/// - `start_x`, `start_z`: world-space coordinates of the chunk's (0,0) column
///
/// Returns `Some(SurfaceNoiseBatch)` with 256 entries per array, or `None` when
/// the GPU path is unavailable (caller falls back to CPU per-column sampling).
pub type SurfaceNoiseGpuFn = fn(
    surface_sampler: &DoublePerlinNoiseSampler,
    secondary_sampler: &DoublePerlinNoiseSampler,
    start_x: i32,
    start_z: i32,
) -> Option<SurfaceNoiseBatch>;

static SURFACE_NOISE_GPU: OnceLock<SurfaceNoiseGpuFn> = OnceLock::new();

/// Register a GPU surface-noise pre-computation function. Call once at server startup.
/// Subsequent calls are no-ops.
pub fn register_surface_noise_gpu(f: SurfaceNoiseGpuFn) {
    let _ = SURFACE_NOISE_GPU.set(f);
}

/// Returns the registered GPU surface-noise function, if any.
#[must_use]
pub fn get_surface_noise_gpu() -> Option<SurfaceNoiseGpuFn> {
    SURFACE_NOISE_GPU.get().copied()
}

pub mod rule;
pub mod terrain;

pub struct MaterialRuleContext<'a> {
    pub min_y: i8,
    pub height: u16,
    pub random_deriver: &'a XoroshiroSplitter,
    fluid_height: i32,
    pub block_pos_x: i32,
    pub block_pos_y: i32,
    pub block_pos_z: i32,
    pub biome: &'a Biome,
    pub run_depth: i32,
    pub secondary_depth: f64,
    packed_chunk_pos: i64,
    estimated_surface_heights: [i32; 4],
    last_unique_horizontal_pos_value: i64,
    last_est_heiht_unique_horizontal_pos_value: i64,
    unique_horizontal_pos_value: i64,
    surface_min_y: i32,
    pub surface_noise: &'a DoublePerlinNoiseSampler,
    pub secondary_noise: &'a DoublePerlinNoiseSampler,
    pub stone_depth_below: i32,
    pub stone_depth_above: i32,
    pub terrain_builder: &'a SurfaceTerrainBuilder,
    pub sea_level: i32,
    steep_material_condition: Option<bool>,
    /// Pre-computed surface noise batch (GPU path), or `None` (CPU path).
    /// Indexed by local column: `noise_batch[lz * 16 + lx]`.
    surface_noise_batch: Option<std::sync::Arc<[f64]>>,
    /// Pre-computed secondary noise batch (GPU path), or `None` (CPU path).
    secondary_noise_batch: Option<std::sync::Arc<[f64]>>,
    /// Chunk start X in world coordinates (used for GPU batch indexing).
    chunk_start_x: i32,
    /// Chunk start Z in world coordinates (used for GPU batch indexing).
    chunk_start_z: i32,
}

impl<'a> MaterialRuleContext<'a> {
    #[must_use]
    pub const fn new(
        min_y: i8,
        height: u16,
        random_deriver: &'a XoroshiroSplitter,
        terrain_builder: &'a SurfaceTerrainBuilder,
        surface_noise: &'a DoublePerlinNoiseSampler,
        secondary_noise: &'a DoublePerlinNoiseSampler,
        sea_level: i32,
    ) -> Self {
        const HORIZONTAL_POS: i64 = -i64::MAX; // Vanilla
        Self {
            min_y,
            height,
            estimated_surface_heights: [0, 0, 0, 0],
            surface_min_y: 0,
            packed_chunk_pos: i64::MAX,
            unique_horizontal_pos_value: HORIZONTAL_POS - 1, // Because pre increment
            last_unique_horizontal_pos_value: HORIZONTAL_POS - 1,
            last_est_heiht_unique_horizontal_pos_value: HORIZONTAL_POS - 1,
            random_deriver,
            terrain_builder,
            fluid_height: 0,
            block_pos_x: 0,
            block_pos_y: 0,
            block_pos_z: 0,
            biome: &Biome::PLAINS,
            run_depth: 0,
            secondary_depth: 0.0,
            surface_noise,
            secondary_noise,
            stone_depth_below: 0,
            stone_depth_above: 0,
            sea_level,
            steep_material_condition: None,
            surface_noise_batch: None,
            secondary_noise_batch: None,
            chunk_start_x: 0,
            chunk_start_z: 0,
        }
    }

    /// Set the pre-computed GPU noise batch for this chunk.
    /// Must be called before `init_horizontal` for any column.
    pub fn set_noise_batch(
        &mut self,
        surface_batch: std::sync::Arc<[f64]>,
        secondary_batch: std::sync::Arc<[f64]>,
        chunk_start_x: i32,
        chunk_start_z: i32,
    ) {
        self.surface_noise_batch = Some(surface_batch);
        self.secondary_noise_batch = Some(secondary_batch);
        self.chunk_start_x = chunk_start_x;
        self.chunk_start_z = chunk_start_z;
    }

    fn sample_run_depth(&self) -> i32 {
        let noise = self.surface_noise_batch.as_ref().map_or_else(
            || {
                self.surface_noise
                    .sample(self.block_pos_x as f64, 0.0, self.block_pos_z as f64)
            },
            |batch| {
                let lx = (self.block_pos_x - self.chunk_start_x) as usize;
                let lz = (self.block_pos_z - self.chunk_start_z) as usize;
                if lx < 16 && lz < 16 {
                    batch[lz * 16 + lx]
                } else {
                    self.surface_noise
                        .sample(self.block_pos_x as f64, 0.0, self.block_pos_z as f64)
                }
            },
        );
        (noise * 2.75
            + 3.0
            + self
                .random_deriver
                .split_pos(self.block_pos_x, 0, self.block_pos_z)
                .next_f64()
                * 0.25) as i32
    }

    pub fn init_horizontal(&mut self, x: i32, z: i32) {
        self.unique_horizontal_pos_value += 1;
        self.block_pos_x = x;
        self.block_pos_z = z;
        self.run_depth = self.sample_run_depth();
    }

    pub const fn init_vertical(
        &mut self,
        stone_depth_above: i32,
        stone_depth_below: i32,
        y: i32,
        fluid_height: i32,
    ) {
        self.block_pos_y = y;
        self.fluid_height = fluid_height;
        self.stone_depth_below = stone_depth_below;
        self.stone_depth_above = stone_depth_above;
    }

    pub fn get_secondary_depth(&mut self) -> f64 {
        if self.last_unique_horizontal_pos_value != self.unique_horizontal_pos_value {
            self.last_unique_horizontal_pos_value = self.unique_horizontal_pos_value;
            self.secondary_depth = if let Some(ref batch) = self.secondary_noise_batch {
                let lx = (self.block_pos_x - self.chunk_start_x) as usize;
                let lz = (self.block_pos_z - self.chunk_start_z) as usize;
                if lx < 16 && lz < 16 {
                    batch[lz * 16 + lx]
                } else {
                    self.secondary_noise.sample(
                        self.block_pos_x as f64,
                        0.0,
                        self.block_pos_z as f64,
                    )
                }
            } else {
                self.secondary_noise
                    .sample(self.block_pos_x as f64, 0.0, self.block_pos_z as f64)
            };
        }
        self.secondary_depth
    }

    pub const fn set_steep_material_condition(&mut self, steep: bool) {
        self.steep_material_condition = Some(steep);
    }
}

pub fn test_condition(
    condition: &MaterialCondition,
    chunk: &mut ProtoChunk,
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> bool {
    match condition {
        MaterialCondition::Biome(biome) => BiomeMaterialCondition::test(biome.biome_is, context),
        MaterialCondition::NoiseThreshold(noise_threshold) => {
            test_noise_threshold(noise_threshold, context)
        }
        MaterialCondition::VerticalGradient(vertical_gradient) => {
            test_vertical_gradient(vertical_gradient, context)
        }
        MaterialCondition::YAbove(above_y) => test_above_y_material(above_y, context),
        MaterialCondition::Water(water) => test_water_material(water, context),
        MaterialCondition::Temperature => {
            let temperature = context.biome.weather.compute_temperature(
                context.block_pos_x as f64,
                context.block_pos_y,
                context.block_pos_z as f64,
                context.sea_level,
            );
            temperature < 0.15f32
        }
        MaterialCondition::Steep => context.steep_material_condition.unwrap_or_else(|| {
            steep_material_condition(chunk, context.block_pos_x, context.block_pos_z)
        }),
        MaterialCondition::Not(not) => {
            test_not_material(not, chunk, context, surface_height_estimate_sampler)
        }
        MaterialCondition::Hole(_hole) => HoleMaterialCondition::test(context),
        MaterialCondition::AbovePreliminarySurface(_above) => {
            SurfaceMaterialCondition::test(context, surface_height_estimate_sampler)
        }
        MaterialCondition::StoneDepth(stone_depth) => test_stone_depth(stone_depth, context),
    }
}

#[must_use]
pub fn steep_material_condition(chunk: &ProtoChunk, block_x: i32, block_z: i32) -> bool {
    let local_x = block_x & 15;
    let local_z = block_z & 15;

    let local_z_sub = 0.max(local_z - 1);
    let local_z_add = 15.min(local_z + 1);

    let sub_height = chunk.top_block_height_exclusive(local_x, local_z_sub);
    let add_height = chunk.top_block_height_exclusive(local_x, local_z_add);

    if add_height >= sub_height + 4 {
        return true;
    }

    let local_x_sub = 0.max(local_x - 1);
    let local_x_add = 15.min(local_x + 1);

    let sub_height = chunk.top_block_height_exclusive(local_x_sub, local_z);
    let add_height = chunk.top_block_height_exclusive(local_x_add, local_z);

    sub_height >= add_height + 4
}

pub struct HoleMaterialCondition;

impl HoleMaterialCondition {
    #[must_use]
    pub const fn test(context: &MaterialRuleContext) -> bool {
        context.run_depth <= 0
    }
}

#[must_use]
pub const fn test_above_y_material(
    condition: &AboveYMaterialCondition,
    context: &MaterialRuleContext,
) -> bool {
    context.block_pos_y
        + if condition.add_stone_depth {
            context.stone_depth_above
        } else {
            0
        }
        >= condition.anchor.get_y(context.min_y as i16, context.height)
            + context.run_depth * condition.surface_depth_multiplier
}

pub fn test_not_material(
    condition: &NotMaterialCondition,
    chunk: &mut ProtoChunk,
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> bool {
    !test_condition(
        condition.invert,
        chunk,
        context,
        surface_height_estimate_sampler,
    )
}

pub struct SurfaceMaterialCondition;

impl SurfaceMaterialCondition {
    pub fn test(
        context: &mut MaterialRuleContext,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) -> bool {
        context.block_pos_y >= estimate_surface_height(context, surface_height_estimate_sampler)
    }
}

pub fn estimate_surface_height(
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> i32 {
    if context.last_est_heiht_unique_horizontal_pos_value != context.unique_horizontal_pos_value {
        context.last_est_heiht_unique_horizontal_pos_value = context.unique_horizontal_pos_value;
        let x = section_coords::block_to_section(context.block_pos_x);
        let z = section_coords::block_to_section(context.block_pos_z);
        let packed = chunk_pos::packed(x as u64, z as u64) as i64;
        if context.packed_chunk_pos != packed {
            context.packed_chunk_pos = packed;
            context.estimated_surface_heights[0] = surface_height_estimate_sampler.estimate_height(
                section_coords::section_to_block(x),
                section_coords::section_to_block(z),
            );
            context.estimated_surface_heights[1] = surface_height_estimate_sampler.estimate_height(
                section_coords::section_to_block(x + 1),
                section_coords::section_to_block(z),
            );
            context.estimated_surface_heights[2] = surface_height_estimate_sampler.estimate_height(
                section_coords::section_to_block(x),
                section_coords::section_to_block(z + 1),
            );
            context.estimated_surface_heights[3] = surface_height_estimate_sampler.estimate_height(
                section_coords::section_to_block(x + 1),
                section_coords::section_to_block(z + 1),
            );
        }
        let surface = lerp2(
            ((context.block_pos_x & 15) as f32 / 16.0) as f64,
            ((context.block_pos_z & 15) as f32 / 16.0) as f64,
            context.estimated_surface_heights[0] as f64,
            context.estimated_surface_heights[1] as f64,
            context.estimated_surface_heights[2] as f64,
            context.estimated_surface_heights[3] as f64,
        )
        .floor() as i32;
        context.surface_min_y = surface.saturating_add(context.run_depth) - 8;
    }
    context.surface_min_y
}

pub struct BiomeMaterialCondition;

impl BiomeMaterialCondition {
    #[must_use]
    pub fn test(biome_is: &[&'static Biome], context: &MaterialRuleContext) -> bool {
        biome_is.contains(&context.biome)
    }
}

pub fn test_noise_threshold(
    condition: &NoiseThresholdMaterialCondition,
    context: &mut MaterialRuleContext,
) -> bool {
    // TODO: we want to cache these
    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(
        context.random_deriver,
        &condition.noise,
    );
    let value = sampler.sample(context.block_pos_x as f64, 0.0, context.block_pos_z as f64);
    value >= condition.min_threshold && value <= condition.max_threshold
}

pub fn test_stone_depth(
    condition: &StoneDepthMaterialCondition,
    context: &mut MaterialRuleContext,
) -> bool {
    let stone_depth = match &condition.surface_type {
        VerticalSurfaceType::Ceiling => context.stone_depth_below,
        VerticalSurfaceType::Floor => context.stone_depth_above,
    };
    let depth = if condition.add_surface_depth {
        context.run_depth
    } else {
        0
    };
    let depth_range = if condition.secondary_depth_range == 0 {
        0
    } else {
        pumpkin_util::math::map(
            context.get_secondary_depth(),
            -1.0,
            1.0,
            0.0,
            condition.secondary_depth_range as f64,
        ) as i32
    };
    stone_depth <= 1 + condition.offset + depth + depth_range
}

#[must_use]
pub const fn test_water_material(
    condition: &WaterMaterialCondition,
    context: &MaterialRuleContext,
) -> bool {
    context.fluid_height == i32::MIN
        || context.block_pos_y
            + (if condition.add_stone_depth {
                context.stone_depth_above
            } else {
                0
            })
            >= context.fluid_height
                + condition.offset
                + context.run_depth * condition.surface_depth_multiplier
}

// random_deriver: ThreadLocal<RefCell<LruCache<usize, RandomDeriver>>>,

#[must_use]
pub fn test_vertical_gradient(
    condition: &VerticalGradientMaterialCondition,
    context: &MaterialRuleContext,
) -> bool {
    let true_at = condition
        .true_at_and_below
        .get_y(context.min_y as i16, context.height);
    let false_at = condition
        .false_at_and_above
        .get_y(context.min_y as i16, context.height);

    let block_y = context.block_pos_y;
    if block_y <= true_at {
        return true;
    }
    if block_y >= false_at {
        return false;
    }
    let splitter = context
        .random_deriver
        .from_lo_and_hi(condition.random_lo, condition.random_hi)
        .next_splitter();
    let mapped = pumpkin_util::math::map(block_y as f32, true_at as f32, false_at as f32, 1.0, 0.0);
    let mut random = splitter.split_pos(context.block_pos_x, block_y, context.block_pos_z);
    random.next_f32() < mapped
}
