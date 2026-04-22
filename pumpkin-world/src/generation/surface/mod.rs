use pumpkin_data::{
    BlockState,
    chunk::{Biome, DoublePerlinNoiseParameters},
    chunk_gen_settings::{ColumnNoiseCache, CompiledSurfaceRule, SurfaceInstruction},
};
use pumpkin_util::{
    math::{lerp2, vertical_surface_type::VerticalSurfaceType},
    random::{RandomImpl, xoroshiro128::XoroshiroSplitter},
    y_offset::YOffset,
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
    pub noise_cache: ColumnNoiseCache,
}

impl<'a> MaterialRuleContext<'a> {
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
            noise_cache: ColumnNoiseCache::new(),
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

pub fn evaluate_surface_rule(
    rule: &CompiledSurfaceRule,
    chunk: &mut ProtoChunk,
    ctx: &mut MaterialRuleContext,
    sampler: &mut SurfaceHeightEstimateSampler,
) -> Option<&'static BlockState> {
    let instrs = &rule.instructions;
    let mut pc = 0usize;
    let x = ctx.block_pos_x;
    let z = ctx.block_pos_z;
    let deriver = ctx.random_deriver;
    while pc < instrs.len() {
        match &instrs[pc] {
            SurfaceInstruction::PlaceBlock { state } => return Some(state),

            SurfaceInstruction::PlaceBadlands => {
                return Some(get_badlands_block(ctx));
            }

            SurfaceInstruction::TestBiome { biome_is, skip } => {
                if !biome_is.iter().any(|b| std::ptr::eq(*b, ctx.biome)) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestNoiseAbove { noise, min, skip } => {
                let v = ctx
                    .noise_cache
                    .get(noise, || sample_noise(noise, deriver, x, z));
                if v < *min {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestNoiseRange {
                noise,
                min,
                max,
                skip,
            } => {
                let v = ctx
                    .noise_cache
                    .get(noise, || sample_noise(noise, deriver, x, z));
                if v < *min || v > *max {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestVerticalGradient {
                random_lo,
                random_hi,
                true_at_and_below,
                false_at_and_above,
                skip,
            } => {
                if !test_vertical_gradient(
                    ctx,
                    *random_lo,
                    *random_hi,
                    true_at_and_below,
                    false_at_and_above,
                ) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestYAbove {
                anchor,
                surface_depth_multiplier,
                add_stone_depth,
                skip,
            } => {
                if !test_y_above(ctx, anchor, *surface_depth_multiplier, *add_stone_depth) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestWater {
                offset,
                surface_depth_multiplier,
                add_stone_depth,
                skip,
            } => {
                if !test_water(ctx, *offset, *surface_depth_multiplier, *add_stone_depth) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestStoneDepth {
                offset,
                add_surface_depth,
                secondary_depth_range,
                surface_type,
                skip,
            } => {
                if !test_stone_depth(
                    ctx,
                    *offset,
                    *add_surface_depth,
                    *secondary_depth_range,
                    surface_type,
                ) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestAbovePreliminarySurface { skip } => {
                if !test_above_preliminary_surface(ctx, sampler) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestHole { skip } => {
                if !test_hole(ctx) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestSteep { skip } => {
                if !test_steep(ctx, chunk) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestTemperature { skip } => {
                if !test_temperature(ctx) {
                    pc += *skip as usize;
                }
            }

            SurfaceInstruction::TestNot { inner, skip } => {
                let passed = eval_single_instruction(inner, chunk, ctx, sampler);
                if passed {
                    pc += *skip as usize;
                }
            }
        }
        pc += 1;
    }
    None
}

#[inline]
fn eval_single_instruction(
    instr: &SurfaceInstruction,
    chunk: &mut ProtoChunk,
    ctx: &mut MaterialRuleContext,
    sampler: &mut SurfaceHeightEstimateSampler,
) -> bool {
    let x = ctx.block_pos_x;
    let z = ctx.block_pos_z;
    let deriver = ctx.random_deriver;
    match instr {
        SurfaceInstruction::TestBiome { biome_is, .. } => {
            biome_is.iter().any(|b| std::ptr::eq(*b, ctx.biome))
        }
        SurfaceInstruction::TestNoiseAbove { noise, min, .. } => {
            ctx.noise_cache
                .get(noise, || sample_noise(noise, deriver, x, z))
                >= *min
        }
        SurfaceInstruction::TestNoiseRange {
            noise, min, max, ..
        } => {
            let v = ctx
                .noise_cache
                .get(noise, || sample_noise(noise, deriver, x, z));
            v >= *min && v <= *max
        }
        SurfaceInstruction::TestHole { .. } => test_hole(ctx),
        SurfaceInstruction::TestSteep { .. } => test_steep(ctx, chunk),
        SurfaceInstruction::TestTemperature { .. } => test_temperature(ctx),
        SurfaceInstruction::TestYAbove {
            anchor,
            surface_depth_multiplier,
            add_stone_depth,
            ..
        } => test_y_above(ctx, anchor, *surface_depth_multiplier, *add_stone_depth),
        SurfaceInstruction::TestWater {
            offset,
            surface_depth_multiplier,
            add_stone_depth,
            ..
        } => test_water(ctx, *offset, *surface_depth_multiplier, *add_stone_depth),
        SurfaceInstruction::TestAbovePreliminarySurface { .. } => {
            test_above_preliminary_surface(ctx, sampler)
        }
        _ => false,
    }
}

#[inline]
pub fn get_badlands_block(context: &MaterialRuleContext) -> &'static BlockState {
    context.terrain_builder.get_terracotta_block(
        context.block_pos_x,
        context.block_pos_y,
        context.block_pos_z,
    )
}

#[inline]
pub const fn test_hole(context: &MaterialRuleContext) -> bool {
    context.run_depth <= 0
}

pub const fn test_y_above(
    context: &MaterialRuleContext,
    anchor: &YOffset,
    surface_depth_multiplier: i32,
    add_stone_depth: bool,
) -> bool {
    context.block_pos_y
        + if add_stone_depth {
            context.stone_depth_above
        } else {
            0
        }
        >= anchor.get_y(context.min_y as i16, context.height)
            + context.run_depth * surface_depth_multiplier
}

#[inline]
pub fn test_above_preliminary_surface(
    context: &mut MaterialRuleContext,
    surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
) -> bool {
    context.block_pos_y >= estimate_surface_height(context, surface_height_estimate_sampler)
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

pub fn sample_noise(
    parameters: &DoublePerlinNoiseParameters,
    random_deriver: &XoroshiroSplitter,
    x: i32,
    z: i32,
) -> f64 {
    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(random_deriver, parameters);
    sampler.sample(x as f64, 0.0, z as f64)
}

pub fn test_stone_depth(
    context: &mut MaterialRuleContext,
    offset: i32,
    add_surface_depth: bool,
    secondary_depth_range: i32,
    surface_type: &VerticalSurfaceType,
) -> bool {
    let stone_depth = match surface_type {
        VerticalSurfaceType::Ceiling => context.stone_depth_below,
        VerticalSurfaceType::Floor => context.stone_depth_above,
    };
    let depth = if add_surface_depth {
        context.run_depth
    } else {
        0
    };
    let depth_range = if secondary_depth_range == 0 {
        0
    } else {
        pumpkin_util::math::map(
            context.get_secondary_depth(),
            -1.0,
            1.0,
            0.0,
            secondary_depth_range as f64,
        ) as i32
    };
    stone_depth <= 1 + offset + depth + depth_range
}

pub const fn test_water(
    context: &MaterialRuleContext,
    offset: i32,
    surface_depth_multiplier: i32,
    add_stone_depth: bool,
) -> bool {
    context.fluid_height == i32::MIN
        || context.block_pos_y
            + (if add_stone_depth {
                context.stone_depth_above
            } else {
                0
            })
            >= context.fluid_height + offset + context.run_depth * surface_depth_multiplier
}

pub fn test_vertical_gradient(
    context: &MaterialRuleContext,
    random_lo: u64,
    random_hi: u64,
    true_at_and_below: &YOffset,
    false_at_and_above: &YOffset,
) -> bool {
    let true_at = true_at_and_below.get_y(context.min_y as i16, context.height);
    let false_at = false_at_and_above.get_y(context.min_y as i16, context.height);

    let block_y = context.block_pos_y;
    if block_y <= true_at {
        return true;
    }
    if block_y >= false_at {
        return false;
    }
    let splitter = context
        .random_deriver
        .from_lo_and_hi(random_lo, random_hi)
        .next_splitter();
    let mapped = pumpkin_util::math::map(block_y as f32, true_at as f32, false_at as f32, 1.0, 0.0);
    let mut random = splitter.split_pos(context.block_pos_x, block_y, context.block_pos_z);
    random.next_f32() < mapped
}

pub fn test_steep(context: &MaterialRuleContext, chunk: &ProtoChunk) -> bool {
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

pub fn test_temperature(context: &MaterialRuleContext) -> bool {
    let temperature = context.biome.weather.compute_temperature(
        context.block_pos_x as f64,
        context.block_pos_y,
        context.block_pos_z as f64,
        context.sea_level,
    );
    temperature < 0.15f32
}
