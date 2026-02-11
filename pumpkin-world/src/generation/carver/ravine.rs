// TODO: Vanilla uses Mth.sin/Mth.cos (65536-entry lookup table) while Pumpkin
// uses standard f32::sin/f32::cos. This causes slight tunnel path divergence.
use std::collections::HashMap;

use pumpkin_util::{
    math::float_provider::FloatProvider,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::generation::positions::chunk_pos;

use super::{Carver, CarverConfig, CarverContext, can_reach, carve_ellipsoid_skip_with_y};

#[derive(Deserialize)]
pub struct RavineCarver {
    #[serde(flatten)]
    pub config: CarverConfig,
    #[serde(rename = "vertical_rotation")]
    pub vertical_rotation: FloatProvider,
    #[serde(default)]
    pub shape: Option<RavineShapeConfig>,
    #[serde(default)]
    pub floor_level: Option<FloatProvider>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct RavineShapeConfig {
    #[serde(rename = "distance_factor")]
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    #[serde(rename = "width_smoothness")]
    pub width_smoothness: i32,
    #[serde(rename = "horizontal_radius_factor")]
    pub horizontal_radius_factor: FloatProvider,
    #[serde(rename = "vertical_radius_default_factor")]
    pub vertical_radius_default_factor: f32,
    #[serde(rename = "vertical_radius_center_factor")]
    pub vertical_radius_center_factor: f32,
}

impl Carver for RavineCarver {
    fn should_carve(&self, random: &mut RandomGenerator) -> bool {
        random.next_f32() <= self.config.probability
    }

    fn carve<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
    ) {
        let range = 4;
        let max_tunnel_length = (range * 2 - 1) * 16;
        let start_x = chunk_pos::start_block_x(context.carver_chunk_pos.x);
        let start_z = chunk_pos::start_block_z(context.carver_chunk_pos.y);
        let x = (start_x + context.random.next_bounded_i32(16)) as f64;
        let y = self
            .config
            .y
            .get(context.random, context.min_y, context.height) as f64;
        let z = (start_z + context.random.next_bounded_i32(16)) as f64;
        let horizontal_angle = context.random.next_f32() * std::f32::consts::TAU;
        let vertical_angle = self.vertical_rotation.get(context.random);
        let y_scale = self.config.y_scale.get(context.random) as f64;
        let shape = self.shape.as_ref();
        let distance_factor = shape
            .map(|config| config.distance_factor.get(context.random))
            .unwrap_or(1.0);
        let thickness = shape
            .map(|config| config.thickness.get(context.random))
            .unwrap_or(1.0);
        let tunnel_length = (max_tunnel_length as f64 * f64::from(distance_factor)) as i32;
        let floor_level = self
            .floor_level
            .as_ref()
            .map(|provider| provider.get(context.random) as f64)
            .unwrap_or(-1.0);

        let tunnel_seed = context.random.next_i64();
        self.create_tunnel(
            context,
            tunnel_seed,
            x,
            y,
            z,
            thickness,
            horizontal_angle,
            vertical_angle,
            0,
            tunnel_length,
            y_scale,
            floor_level,
        );
    }
}

struct RavineTunnelState {
    x: f64,
    y: f64,
    z: f64,
    horizontal_angle: f32,
    vertical_angle: f32,
    yaw_delta: f32,
    pitch_delta: f32,
}

impl RavineTunnelState {
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
        rand: &mut RandomGenerator,
    ) -> (f64, f64) {
        let horizontal_radius =
            1.5 + (std::f64::consts::PI * step as f64 / end_step as f64).sin() * thickness as f64;
        let vertical_radius = horizontal_radius * y_scale;
        let y_cos = self.vertical_angle.cos();
        let y_sin = self.vertical_angle.sin();
        self.x += (self.horizontal_angle.cos() * y_cos) as f64;
        self.y += y_sin as f64;
        self.z += (self.horizontal_angle.sin() * y_cos) as f64;
        self.vertical_angle *= 0.7;
        self.vertical_angle += self.pitch_delta * 0.05;
        self.horizontal_angle += self.yaw_delta * 0.05;
        self.pitch_delta *= 0.8;
        self.yaw_delta *= 0.5;
        self.pitch_delta += (rand.next_f32() - rand.next_f32()) * rand.next_f32() * 2.0;
        self.yaw_delta += (rand.next_f32() - rand.next_f32()) * rand.next_f32() * 4.0;
        (horizontal_radius, vertical_radius)
    }
}

impl RavineCarver {
    #[allow(clippy::too_many_arguments)]
    fn create_tunnel<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &mut CarverContext<'_, '_, T>,
        seed: i64,
        x: f64,
        y: f64,
        z: f64,
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
        let width_factors = self.init_width_factors(context, &mut rand);
        let mut state = RavineTunnelState::new(x, y, z, horizontal_angle, vertical_angle);

        for step in start_step..end_step {
            let (mut horizontal_radius, mut vertical_radius) =
                state.advance(step, end_step, thickness, y_scale, &mut rand);
            if let Some(shape) = self.shape.as_ref() {
                horizontal_radius *= shape.horizontal_radius_factor.get(&mut rand) as f64;
                vertical_radius = self.update_vertical_radius(
                    &mut rand,
                    vertical_radius,
                    end_step as f32,
                    step as f32,
                );
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
                    horizontal_radius,
                    vertical_radius,
                    &width_factors,
                    floor_level,
                );
            }
        }
    }

    fn init_width_factors<T: crate::generation::proto_chunk::GenerationCache>(
        &self,
        context: &CarverContext<'_, '_, T>,
        random: &mut RandomGenerator,
    ) -> Vec<f32> {
        let depth = context.height as usize;
        let mut factors = vec![0.0f32; depth];
        let mut current = 1.0f32;
        let width_smoothness = self
            .shape
            .as_ref()
            .map(|config| config.width_smoothness)
            .unwrap_or(1);

        for (index, factor) in factors.iter_mut().enumerate() {
            if index == 0 || random.next_bounded_i32(width_smoothness.max(1)) == 0 {
                current = 1.0 + random.next_f32() * random.next_f32();
            }
            *factor = current * current;
        }

        factors
    }

    fn update_vertical_radius(
        &self,
        random: &mut RandomGenerator,
        radius: f64,
        total_steps: f32,
        step: f32,
    ) -> f64 {
        let Some(shape) = self.shape.as_ref() else {
            return radius;
        };
        let t = 1.0 - (0.5 - step / total_steps).abs() * 2.0;
        let factor = shape.vertical_radius_default_factor + shape.vertical_radius_center_factor * t;
        let random_factor = 0.75 + random.next_f32() * 0.25;
        factor as f64 * radius * random_factor as f64
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
        width_factors: &[f32],
        _floor_level: f64,
    ) {
        let min_y = context.min_y as i32;
        let skip_checker = |dx: f64, dy: f64, dz: f64, y: i32| -> bool {
            let index = (y - min_y) as usize;
            if index == 0 || index >= width_factors.len() {
                return true;
            }
            (dx * dx + dz * dz) * width_factors[index - 1] as f64 + dy * dy / 6.0 >= 1.0
        };
        carve_ellipsoid_skip_with_y(
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
