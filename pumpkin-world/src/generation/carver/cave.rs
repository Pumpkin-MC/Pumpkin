// TODO: Vanilla uses Mth.sin/Mth.cos (65536-entry lookup table) while Pumpkin
// uses standard f32::sin/f32::cos. This causes slight tunnel path divergence.
use std::collections::HashMap;

use pumpkin_util::{
    math::float_provider::FloatProvider,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::generation::positions::chunk_pos;

use super::{Carver, CarverConfig, CarverContext, carve_ellipsoid_skip};

#[derive(Deserialize)]
pub struct CaveCarver {
    #[serde(flatten)]
    pub config: CarverConfig,
    #[serde(default)]
    pub horizontal_radius_multiplier: Option<pumpkin_util::math::float_provider::FloatProvider>,
    #[serde(default)]
    pub vertical_radius_multiplier: Option<pumpkin_util::math::float_provider::FloatProvider>,
    #[serde(default)]
    pub floor_level: Option<pumpkin_util::math::float_provider::FloatProvider>,
    #[serde(default, rename = "horizontal_rotation")]
    pub horizontal_rotation: Option<FloatProvider>,
    #[serde(default, rename = "vertical_rotation")]
    pub vertical_rotation: Option<FloatProvider>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Carver for CaveCarver {
    fn should_carve(&self, random: &mut RandomGenerator) -> bool {
        random.next_f32() <= self.config.probability
    }

    fn carve<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
    ) {
        let range = 4;
        let max_tunnel_length = (range * 2 - 1) * 16;
        let mut cave_count = context.random.next_bounded_i32(self.get_cave_bound());
        cave_count = context.random.next_bounded_i32(cave_count + 1);
        cave_count = context.random.next_bounded_i32(cave_count + 1);

        if cave_count <= 0 {
            return;
        }

        let start_x = chunk_pos::start_block_x(context.carver_chunk_pos.x);
        let start_z = chunk_pos::start_block_z(context.carver_chunk_pos.y);

        for _ in 0..cave_count {
            let x = start_x + context.random.next_bounded_i32(16);
            let z = start_z + context.random.next_bounded_i32(16);
            let y = self
                .config
                .y
                .get(context.random, context.min_y, context.height) as f64;
            let horizontal_multiplier = self
                .horizontal_radius_multiplier
                .as_ref()
                .map(|provider| provider.get(context.random) as f64)
                .unwrap_or(1.0);
            let vertical_multiplier = self
                .vertical_radius_multiplier
                .as_ref()
                .map(|provider| provider.get(context.random) as f64)
                .unwrap_or(1.0);
            let floor_level = self
                .floor_level
                .as_ref()
                .map(|provider| provider.get(context.random) as f64)
                .unwrap_or(-1.0);
            let horizontal_rotation = self
                .horizontal_rotation
                .as_ref()
                .map(|provider| provider.get(context.random))
                .unwrap_or(0.0);
            let vertical_rotation = self
                .vertical_rotation
                .as_ref()
                .map(|provider| provider.get(context.random))
                .unwrap_or(0.0);

            let mut tunnel_count = 1;
            if context.random.next_bounded_i32(4) == 0 {
                let y_scale = self.config.y_scale.get(context.random) as f64;
                let room_radius = 1.0 + context.random.next_f32() * 6.0;
                self.create_room(
                    context,
                    x as f64,
                    y,
                    z as f64,
                    room_radius,
                    y_scale,
                    floor_level,
                );
                tunnel_count += context.random.next_bounded_i32(4);
            }

            for _ in 0..tunnel_count {
                let horizontal_angle =
                    context.random.next_f32() * std::f32::consts::TAU + horizontal_rotation;
                let vertical_angle = (context.random.next_f32() - 0.5) / 4.0 + vertical_rotation;
                let thickness = self.get_thickness(context.random);
                let tunnel_length = max_tunnel_length
                    - context
                        .random
                        .next_bounded_i32((max_tunnel_length / 4).max(1));

                let tunnel_seed = context.random.next_i64();
                self.create_tunnel(
                    context,
                    tunnel_seed,
                    x as f64,
                    y,
                    z as f64,
                    horizontal_multiplier,
                    vertical_multiplier,
                    thickness,
                    horizontal_angle,
                    vertical_angle,
                    0,
                    tunnel_length,
                    self.get_y_scale(),
                    floor_level,
                );
            }
        }
    }
}

struct CaveTunnelState {
    x: f64,
    y: f64,
    z: f64,
    horizontal_angle: f32,
    vertical_angle: f32,
    yaw_delta: f32,
    pitch_delta: f32,
}

impl CaveTunnelState {
    fn new(x: f64, y: f64, z: f64, horizontal_angle: f32, vertical_angle: f32) -> Self {
        Self {
            x,
            y,
            z,
            horizontal_angle,
            vertical_angle,
            yaw_delta: 0.0,
            pitch_delta: 0.0,
        }
    }

    fn advance(
        &mut self,
        step: i32,
        end_step: i32,
        thickness: f32,
        y_scale: f64,
        steep: bool,
        rand: &mut RandomGenerator,
    ) -> (f64, f64) {
        let radius =
            1.5 + (std::f64::consts::PI * step as f64 / end_step as f64).sin() * thickness as f64;
        let vertical_radius = radius * y_scale;
        let y_cos = self.vertical_angle.cos();
        self.x += (self.horizontal_angle.cos() * y_cos) as f64;
        self.y += self.vertical_angle.sin() as f64;
        self.z += (self.horizontal_angle.sin() * y_cos) as f64;
        self.vertical_angle *= if steep { 0.92 } else { 0.7 };
        self.vertical_angle += self.pitch_delta * 0.1;
        self.horizontal_angle += self.yaw_delta * 0.1;
        self.pitch_delta *= 0.9;
        self.yaw_delta *= 0.75;
        self.pitch_delta += (rand.next_f32() - rand.next_f32()) * rand.next_f32() * 2.0;
        self.yaw_delta += (rand.next_f32() - rand.next_f32()) * rand.next_f32() * 4.0;
        (radius, vertical_radius)
    }
}

impl CaveCarver {
    pub(crate) fn get_cave_bound(&self) -> i32 {
        if self.config.replaceable.contains("nether") {
            10
        } else {
            15
        }
    }

    pub(crate) fn get_thickness(&self, random: &mut RandomGenerator) -> f32 {
        let mut thickness = random.next_f32() * 2.0 + random.next_f32();
        if self.config.replaceable.contains("nether") {
            thickness *= 2.0;
            return thickness;
        }
        if random.next_bounded_i32(10) == 0 {
            thickness *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
        }
        thickness
    }

    pub(crate) fn get_y_scale(&self) -> f64 {
        if self.config.replaceable.contains("nether") {
            5.0
        } else {
            1.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_room<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
        center_x: f64,
        center_y: f64,
        center_z: f64,
        radius: f32,
        y_scale: f64,
        floor_level: f64,
    ) {
        let horizontal_radius = 1.5 + (std::f32::consts::FRAC_PI_2.sin() as f64 * radius as f64);
        let vertical_radius = horizontal_radius * y_scale;
        self.carve_ellipsoid(
            context,
            center_x + 1.0,
            center_y,
            center_z,
            horizontal_radius,
            vertical_radius,
            floor_level,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn create_tunnel<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
        seed: i64,
        x: f64,
        y: f64,
        z: f64,
        horizontal_multiplier: f64,
        vertical_multiplier: f64,
        thickness: f32,
        horizontal_angle: f32,
        vertical_angle: f32,
        start_step: i32,
        end_step: i32,
        y_scale: f64,
        floor_level: f64,
    ) {
        let mut rand = RandomGenerator::Legacy(
            pumpkin_util::random::legacy_rand::LegacyRand::from_seed(seed as u64),
        );
        let split_step = rand.next_bounded_i32(end_step / 2) + end_step / 4;
        let steep = rand.next_bounded_i32(6) == 0;
        let mut state = CaveTunnelState::new(x, y, z, horizontal_angle, vertical_angle);

        for step in start_step..end_step {
            let (radius, vertical_radius) =
                state.advance(step, end_step, thickness, y_scale, steep, &mut rand);

            if step == split_step && thickness > 1.0 {
                self.create_tunnel(
                    context,
                    rand.next_i64(),
                    state.x,
                    state.y,
                    state.z,
                    horizontal_multiplier,
                    vertical_multiplier,
                    rand.next_f32() * 0.5 + 0.5,
                    state.horizontal_angle - std::f32::consts::FRAC_PI_2,
                    state.vertical_angle / 3.0,
                    step,
                    end_step,
                    1.0,
                    floor_level,
                );
                self.create_tunnel(
                    context,
                    rand.next_i64(),
                    state.x,
                    state.y,
                    state.z,
                    horizontal_multiplier,
                    vertical_multiplier,
                    rand.next_f32() * 0.5 + 0.5,
                    state.horizontal_angle + std::f32::consts::FRAC_PI_2,
                    state.vertical_angle / 3.0,
                    step,
                    end_step,
                    1.0,
                    floor_level,
                );
                return;
            }

            if rand.next_bounded_i32(4) != 0 {
                if !can_reach(
                    context.chunk_pos,
                    state.x,
                    state.z,
                    step,
                    end_step,
                    thickness,
                ) {
                    return;
                }
                self.carve_ellipsoid(
                    context,
                    state.x,
                    state.y,
                    state.z,
                    radius * horizontal_multiplier,
                    vertical_radius * vertical_multiplier,
                    floor_level,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn carve_ellipsoid<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
        center_x: f64,
        center_y: f64,
        center_z: f64,
        horizontal_radius: f64,
        vertical_radius: f64,
        floor_level: f64,
    ) {
        let skip_checker = |dx: f64, dy: f64, dz: f64| -> bool {
            if dy <= floor_level {
                true
            } else {
                dx * dx + dy * dy + dz * dz >= 1.0
            }
        };
        carve_ellipsoid_skip(
            context,
            &self.config.replaceable,
            center_x,
            center_y,
            center_z,
            horizontal_radius,
            vertical_radius,
            skip_checker,
        );
    }
}

pub(crate) fn can_reach(
    chunk_pos: pumpkin_util::math::vector2::Vector2<i32>,
    x: f64,
    z: f64,
    start_step: i32,
    end_step: i32,
    thickness: f32,
) -> bool {
    let center_x = chunk_pos::start_block_x(chunk_pos.x) as f64 + 8.0;
    let center_z = chunk_pos::start_block_z(chunk_pos.y) as f64 + 8.0;
    let dx = x - center_x;
    let dz = z - center_z;
    let remaining = (end_step - start_step) as f64;
    let radius = thickness as f64 + 2.0 + 16.0;
    dx * dx + dz * dz - remaining * remaining <= radius * radius
}
