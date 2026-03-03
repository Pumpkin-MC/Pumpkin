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
    random::{RandomDeriver, RandomDeriverImpl, RandomImpl},
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

pub mod rule;
pub mod terrain;

pub struct MaterialRuleContext<'a> {
    pub min_y: i8,
    pub height: u16,
    pub random_deriver: &'a RandomDeriver,
    fluid_height: i32,
    pub block_pos_x: i32,
    pub block_pos_y: i32,
    pub block_pos_z: i32,
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
    pub surface_noise: &'a DoublePerlinNoiseSampler,
    pub secondary_noise: &'a DoublePerlinNoiseSampler,
    pub stone_depth_below: i32,
    pub stone_depth_above: i32,
    pub terrain_builder: &'a SurfaceTerrainBuilder,
    pub sea_level: i32,
}

impl<'a> MaterialRuleContext<'a> {
    #[expect(clippy::too_many_arguments)]
    pub const fn new(
        min_y: i8,
        height: u16,
        noise_builder: DoublePerlinNoiseBuilder<'a>,
        random_deriver: &'a RandomDeriver,
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
            noise_builder,
            stone_depth_below: 0,
            stone_depth_above: 0,
            sea_level,
        }
    }

    fn sample_run_depth(&self) -> i32 {
        let noise =
            self.surface_noise
                .sample(self.block_pos_x as f64, 0.0, self.block_pos_z as f64);
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
            self.secondary_depth =
                self.secondary_noise
                    .sample(self.block_pos_x as f64, 0.0, self.block_pos_z as f64);
        }
        self.secondary_depth
    }
}

#[expect(clippy::similar_names)]
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
        MaterialCondition::Steep => {
            let local_x = context.block_pos_x & 15;
            let local_z = context.block_pos_z & 15;

            let local_z_sub = 0.max(local_z - 1);
            let local_z_add = 15.min(local_z + 1);

            let sub_height = chunk.top_block_height_exclusive(local_x, local_z_sub);
            let add_height = chunk.top_block_height_exclusive(local_x, local_z_add);

            if add_height >= sub_height + 4 {
                true
            } else {
                let local_x_sub = 0.max(local_x - 1);
                let local_x_add = 15.min(local_x + 1);

                let sub_height = chunk.top_block_height_exclusive(local_x_sub, local_z);
                let add_height = chunk.top_block_height_exclusive(local_x_add, local_z);

                sub_height >= add_height + 4
            }
        }
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

pub struct HoleMaterialCondition;

impl HoleMaterialCondition {
    pub const fn test(context: &MaterialRuleContext) -> bool {
        context.run_depth <= 0
    }
}

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
        // TODO
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
    pub fn test(biome_is: &[&'static Biome], context: &MaterialRuleContext) -> bool {
        biome_is.contains(&context.biome)
    }
}

pub fn test_noise_threshold(
    condition: &NoiseThresholdMaterialCondition,
    context: &mut MaterialRuleContext,
) -> bool {
    // TODO: we want to cache these
    let sampler = context
        .noise_builder
        .get_noise_sampler_for_id(condition.noise.strip_prefix("minecraft:").unwrap());
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

    let splitter = context
        .random_deriver
        .split_string(condition.random_name)
        .next_splitter();

    let block_y = context.block_pos_y;
    if block_y <= true_at {
        return true;
    }
    if block_y >= false_at {
        return false;
    }
    let mapped = pumpkin_util::math::map(block_y as f32, true_at as f32, false_at as f32, 1.0, 0.0);
    let mut random = splitter.split_pos(context.block_pos_x, block_y, context.block_pos_z);
    random.next_f32() < mapped
}
