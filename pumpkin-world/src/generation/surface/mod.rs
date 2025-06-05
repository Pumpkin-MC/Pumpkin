use std::{cell::RefCell, num::NonZeroUsize};

use lru::LruCache;
use pumpkin_data::chunk::Biome;
use pumpkin_util::{
    math::{lerp2, vector2::Vector2, vector3::Vector3, vertical_surface_type::VerticalSurfaceType},
    random::{RandomDeriver, RandomDeriverImpl, RandomImpl},
};
use serde::Deserialize;

use terrain::SurfaceTerrainBuilder;
use thread_local::ThreadLocal;

use crate::{
    ProtoChunk,
    generation::{positions::chunk_pos, section_coords},
};

use super::{
    noise::perlin::DoublePerlinNoiseSampler,
    noise_router::{
        proto_noise_router::DoublePerlinNoiseBuilder,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
    y_offset::YOffset,
};

pub mod rule;
pub mod terrain;

pub struct MaterialRuleContext<'a> {
    pub min_y: i8,
    pub height: u16,
    pub random_deriver: &'a RandomDeriver,
    fluid_height: i32,
    pub block_pos: Vector3<i32>,
    pub biome: &'a Biome,
    pub run_depth: i32,
    pub secondary_depth: f64,
    noise_builder: DoublePerlinNoiseBuilder<'a>,
    packed_chunk_pos: i64,
    estimated_surface_heights: [i32; 4],
    last_unique_horizontal_pos_value: i64,
    last_est_heiht_unique_horizontal_pos_value: i64,
    unique_horizontal_pos_value: i64,
    surface_min_y: i32,
    pub surface_noise: DoublePerlinNoiseSampler,
    pub secondary_noise: DoublePerlinNoiseSampler,
    pub stone_depth_below: i32,
    pub stone_depth_above: i32,
    pub terrain_builder: &'a SurfaceTerrainBuilder,
}

impl<'a> MaterialRuleContext<'a> {
    pub fn new(
        min_y: i8,
        height: u16,
        mut noise_builder: DoublePerlinNoiseBuilder<'a>,
        random_deriver: &'a RandomDeriver,
        terrain_builder: &'a SurfaceTerrainBuilder,
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
            block_pos: Vector3::new(0, 0, 0),
            biome: &Biome::PLAINS,
            run_depth: 0,
            secondary_depth: 0.0,
            surface_noise: noise_builder.get_noise_sampler_for_id("surface"),
            secondary_noise: noise_builder.get_noise_sampler_for_id("surface_secondary"),
            noise_builder,
            stone_depth_below: 0,
            stone_depth_above: 0,
        }
    }

    fn sample_run_depth(&self) -> i32 {
        let noise =
            self.surface_noise
                .sample(self.block_pos.x as f64, 0.0, self.block_pos.z as f64);
        (noise * 2.75
            + 3.0
            + self
                .random_deriver
                .split_pos(self.block_pos.x, 0, self.block_pos.z)
                .next_f64()
                * 0.25) as i32
    }

    pub fn init_horizontal(&mut self, x: i32, z: i32) {
        self.unique_horizontal_pos_value += 1;
        self.block_pos.x = x;
        self.block_pos.z = z;
        self.run_depth = self.sample_run_depth();
    }

    pub fn init_vertical(
        &mut self,
        stone_depth_above: i32,
        stone_depth_below: i32,
        y: i32,
        fluid_height: i32,
    ) {
        self.block_pos.y = y;
        self.fluid_height = fluid_height;
        self.stone_depth_below = stone_depth_below;
        self.stone_depth_above = stone_depth_above;
    }

    pub fn get_secondary_depth(&mut self) -> f64 {
        if self.last_unique_horizontal_pos_value != self.unique_horizontal_pos_value {
            self.last_unique_horizontal_pos_value = self.unique_horizontal_pos_value;
            self.secondary_depth =
                self.secondary_noise
                    .sample(self.block_pos.x as f64, 0.0, self.block_pos.z as f64)
        }
        self.secondary_depth
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MaterialCondition {
    #[serde(rename = "minecraft:biome")]
    Biome(BiomeMaterialCondition),
    #[serde(rename = "minecraft:noise_threshold")]
    NoiseThreshold(NoiseThresholdMaterialCondition),
    #[serde(rename = "minecraft:vertical_gradient")]
    VerticalGradient(Box<VerticalGradientMaterialCondition>),
    #[serde(rename = "minecraft:y_above")]
    YAbove(AboveYMaterialCondition),
    #[serde(rename = "minecraft:water")]
    Water(WaterMaterialCondition),
    #[serde(rename = "minecraft:temperature")]
    Temperature,
    #[serde(rename = "minecraft:steep")]
    Steep,
    #[serde(rename = "minecraft:not")]
    Not(NotMaterialCondition),
    #[serde(rename = "minecraft:hole")]
    Hole(HoleMaterialCondition),
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface(SurfaceMaterialCondition),
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth(StoneDepthMaterialCondition),
}

impl MaterialCondition {
    pub fn test(&self, chunk: &mut ProtoChunk, context: &mut MaterialRuleContext) -> bool {
        match self {
            MaterialCondition::Biome(biome) => biome.test(context),
            MaterialCondition::NoiseThreshold(noise_threshold) => noise_threshold.test(context),
            MaterialCondition::VerticalGradient(vertical_gradient) => {
                vertical_gradient.test(context)
            }
            MaterialCondition::YAbove(above_y) => above_y.test(context),
            MaterialCondition::Water(water) => water.test(context),
            MaterialCondition::Temperature => {
                let temperature = context
                    .biome
                    .weather
                    .compute_temperature(&context.block_pos, chunk.generation_settings().sea_level);
                temperature < 0.15f32
            }
            MaterialCondition::Steep => {
                let local_x = context.block_pos.x & 15;
                let local_z = context.block_pos.z & 15;

                let local_z_sub = 0.max(local_z - 1);
                let local_z_add = 15.min(local_z + 1);

                let sub_height =
                    chunk.top_block_height_exclusive(&Vector2::new(local_x, local_z_sub));
                let add_height =
                    chunk.top_block_height_exclusive(&Vector2::new(local_x, local_z_add));

                if add_height >= sub_height + 4 {
                    true
                } else {
                    let local_x_sub = 0.max(local_x - 1);
                    let local_x_add = 15.min(local_x + 1);

                    let sub_height =
                        chunk.top_block_height_exclusive(&Vector2::new(local_x_sub, local_z));
                    let add_height =
                        chunk.top_block_height_exclusive(&Vector2::new(local_x_add, local_z));

                    sub_height >= add_height + 4
                }
            }
            MaterialCondition::Not(not) => not.test(chunk, context),
            MaterialCondition::Hole(hole) => hole.test(context),
            MaterialCondition::AbovePreliminarySurface(above) => {
                above.test(context, &mut chunk.surface_height_estimate_sampler)
            }
            MaterialCondition::StoneDepth(stone_depth) => stone_depth.test(context),
        }
    }
}

#[derive(Deserialize)]
pub struct HoleMaterialCondition;

impl HoleMaterialCondition {
    pub fn test(&self, context: &MaterialRuleContext) -> bool {
        context.run_depth <= 0
    }
}

#[derive(Deserialize)]
pub struct AboveYMaterialCondition {
    anchor: YOffset,
    surface_depth_multiplier: i32,
    add_stone_depth: bool,
}

impl AboveYMaterialCondition {
    pub fn test(&self, context: &MaterialRuleContext) -> bool {
        context.block_pos.y
            + if self.add_stone_depth {
                context.stone_depth_above
            } else {
                0
            }
            >= self.anchor.get_y(context.min_y, context.height) as i32
                + context.run_depth * self.surface_depth_multiplier
    }
}

#[derive(Deserialize)]
pub struct NotMaterialCondition {
    invert: Box<MaterialCondition>,
}

impl NotMaterialCondition {
    pub fn test(&self, chunk: &mut ProtoChunk, context: &mut MaterialRuleContext) -> bool {
        !self.invert.test(chunk, context)
    }
}

#[derive(Deserialize)]
pub struct SurfaceMaterialCondition;

impl SurfaceMaterialCondition {
    pub fn test(
        &self,
        context: &mut MaterialRuleContext,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) -> bool {
        // TODO
        context.block_pos.y >= estimate_surface_height(context, surface_height_estimate_sampler)
    }
}

pub fn estimate_surface_height(
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> i32 {
    if context.last_est_heiht_unique_horizontal_pos_value != context.unique_horizontal_pos_value {
        context.last_est_heiht_unique_horizontal_pos_value = context.unique_horizontal_pos_value;
        let x = section_coords::block_to_section(context.block_pos.x);
        let z = section_coords::block_to_section(context.block_pos.z);
        let packed = chunk_pos::packed(&Vector2::new(x, z)) as i64;
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
            ((context.block_pos.x & 15) as f32 / 16.0) as f64,
            ((context.block_pos.z & 15) as f32 / 16.0) as f64,
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

#[derive(Deserialize)]
pub struct BiomeMaterialCondition {
    biome_is: Box<[&'static Biome]>,
}

impl BiomeMaterialCondition {
    pub fn test(&self, context: &MaterialRuleContext) -> bool {
        self.biome_is.contains(&context.biome)
    }
}

#[derive(Deserialize)]
pub struct NoiseThresholdMaterialCondition {
    noise: String,
    min_threshold: f64,
    max_threshold: f64,
}

impl NoiseThresholdMaterialCondition {
    pub fn test(&self, context: &mut MaterialRuleContext) -> bool {
        let sampler = context
            .noise_builder
            .get_noise_sampler_for_id(&self.noise.replace("minecraft:", ""));
        let value = sampler.sample(context.block_pos.x as f64, 0.0, context.block_pos.z as f64);
        value >= self.min_threshold && value <= self.max_threshold
    }
}

#[derive(Deserialize)]
pub struct StoneDepthMaterialCondition {
    offset: i32,
    add_surface_depth: bool,
    secondary_depth_range: i32,
    surface_type: VerticalSurfaceType,
}

impl StoneDepthMaterialCondition {
    pub fn test(&self, context: &mut MaterialRuleContext) -> bool {
        let stone_depth = match &self.surface_type {
            VerticalSurfaceType::Ceiling => context.stone_depth_below,
            VerticalSurfaceType::Floor => context.stone_depth_above,
        };
        let depth = if self.add_surface_depth {
            context.run_depth
        } else {
            0
        };
        let depth_range = if self.secondary_depth_range == 0 {
            0
        } else {
            pumpkin_util::math::map(
                context.get_secondary_depth(),
                -1.0,
                1.0,
                0.0,
                self.secondary_depth_range as f64,
            ) as i32
        };
        stone_depth <= 1 + self.offset + depth + depth_range
    }
}

#[derive(Deserialize)]
pub struct WaterMaterialCondition {
    offset: i32,
    surface_depth_multiplier: i32,
    add_stone_depth: bool,
}

impl WaterMaterialCondition {
    pub fn test(&self, context: &MaterialRuleContext) -> bool {
        context.fluid_height == i32::MIN
            || context.block_pos.y
                + (if self.add_stone_depth {
                    context.stone_depth_above
                } else {
                    0
                })
                >= context.fluid_height
                    + self.offset
                    + context.run_depth * self.surface_depth_multiplier
    }
}

#[derive(Deserialize)]
pub struct VerticalGradientMaterialCondition {
    random_name: String,
    true_at_and_below: YOffset,
    false_at_and_above: YOffset,
    #[serde(skip)]
    random_deriver: ThreadLocal<RefCell<LruCache<usize, RandomDeriver>>>,
}

impl VerticalGradientMaterialCondition {
    pub fn test(&self, context: &MaterialRuleContext) -> bool {
        let true_at = self.true_at_and_below.get_y(context.min_y, context.height);
        let false_at = self.false_at_and_above.get_y(context.min_y, context.height);

        let context_pointer: *const RandomDeriver = context.random_deriver;
        let key = context_pointer.addr();

        let mut cache = self
            .random_deriver
            .get_or(|| {
                let cache_size = NonZeroUsize::new(128).unwrap();
                let cache = LruCache::new(cache_size);
                RefCell::new(cache)
            })
            .borrow_mut();

        let splitter = cache.get_or_insert(key, || {
            context
                .random_deriver
                .split_string(&self.random_name)
                .next_splitter()
        });

        let block_y = context.block_pos.y;
        if block_y <= true_at as i32 {
            return true;
        }
        if block_y >= false_at as i32 {
            return false;
        }
        let mapped =
            pumpkin_util::math::map(block_y as f32, true_at as f32, false_at as f32, 1.0, 0.0);
        let mut random = splitter.split_pos(context.block_pos.x, block_y, context.block_pos.z);
        random.next_f32() < mapped
    }
}
