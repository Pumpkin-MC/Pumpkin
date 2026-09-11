use super::cave::get_height;
use super::{Carver, CarverOutput};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use std::f32::consts::PI;

pub struct CanyonCarver;

impl Carver for CanyonCarver {
    fn carve(
        &self,
        config: &CarverConfig,
        output: &mut dyn CarverOutput,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        min_gen_y: i8,
        gen_depth: u16,
        legacy_random_source: bool,
    ) {
        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return;
        };

        let max_distance = (4 * 2 - 1) * 16;

        let x = (carver_chunk_pos.x << 4) + random.next_bounded_i32(16);
        let y = get_height(&config.y, random, min_gen_y, gen_depth);
        let z = (carver_chunk_pos.y << 4) + random.next_bounded_i32(16);

        let horizontal_rotation = random.next_f32() * PI * 2.0;
        let vertical_rotation = canyon_config.vertical_rotation.get(random);
        let y_scale = canyon_config.shape.y_scale.get(random) as f64;
        let thickness = canyon_config.shape.thickness.get(random);
        let distance =
            (max_distance as f32 * canyon_config.shape.distance_factor.get(random)) as i32;
        let tunnel_seed = random.next_i64();

        Self::do_carve(
            config,
            *chunk_pos,
            output,
            min_gen_y,
            gen_depth,
            tunnel_seed,
            x as f64,
            y as f64,
            z as f64,
            thickness,
            horizontal_rotation,
            vertical_rotation,
            0,
            distance,
            y_scale,
            legacy_random_source,
        );
    }
}

impl CanyonCarver {
    #[allow(clippy::too_many_arguments)]
    fn do_carve(
        config: &CarverConfig,
        chunk_pos: Vector2<i32>,
        output: &mut dyn CarverOutput,
        min_gen_y: i8,
        gen_depth: u16,
        tunnel_seed: i64,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        thickness: f32,
        mut horizontal_rotation: f32,
        mut vertical_rotation: f32,
        step: i32,
        distance: i32,
        y_scale: f64,
        legacy_random_source: bool,
    ) {
        let mut random = super::new_carver_random(tunnel_seed as u64, legacy_random_source);
        let width_factor_per_height =
            Self::init_width_factors(gen_depth as usize, config, &mut random);
        let mut y_rota = 0.0f32;
        let mut x_rota = 0.0f32;

        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return;
        };

        for current_step in step..distance {
            let progress = current_step as f32 * PI / distance as f32;
            let mut horizontal_radius =
                1.5 + f64::from(pumpkin_util::math::sin(progress) * thickness);
            let mut vertical_radius = horizontal_radius * y_scale;
            horizontal_radius *= canyon_config
                .shape
                .horizontal_radius_factor
                .get(&mut random) as f64;
            vertical_radius = Self::update_vertical_radius(
                config,
                &mut random,
                vertical_radius,
                distance as f32,
                current_step as f32,
            );

            let xc = pumpkin_util::math::cos(vertical_rotation);
            let xs = pumpkin_util::math::sin(vertical_rotation);
            x += f64::from(pumpkin_util::math::cos(horizontal_rotation) * xc);
            y += xs as f64;
            z += f64::from(pumpkin_util::math::sin(horizontal_rotation) * xc);

            vertical_rotation = (vertical_rotation * 0.7) + (x_rota * 0.05);
            horizontal_rotation += y_rota * 0.05;
            x_rota = (x_rota * 0.8)
                + ((random.next_f32() - random.next_f32()) * random.next_f32() * 2.0);
            y_rota = (y_rota * 0.5)
                + ((random.next_f32() - random.next_f32()) * random.next_f32() * 4.0);

            if random.next_bounded_i32(4) != 0 {
                if !super::can_reach(&chunk_pos, x, z, current_step, distance, thickness) {
                    return;
                }

                super::carve_ellipsoid(
                    &chunk_pos,
                    x,
                    y,
                    z,
                    horizontal_radius,
                    vertical_radius,
                    output,
                    |xd, yd, zd, world_y| {
                        Self::should_skip(
                            &width_factor_per_height,
                            xd,
                            yd,
                            zd,
                            world_y,
                            min_gen_y as i32,
                        )
                    },
                );
            }
        }
    }

    fn init_width_factors(
        depth: usize,
        config: &CarverConfig,
        random: &mut RandomGenerator,
    ) -> Vec<f32> {
        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return vec![1.0; depth];
        };
        let mut width_factor_per_height = vec![0.0; depth];
        let mut width_factor = 1.0f32;

        for (y_index, item) in width_factor_per_height.iter_mut().enumerate() {
            if y_index == 0 || random.next_bounded_i32(canyon_config.shape.width_smoothness) == 0 {
                width_factor = 1.0 + random.next_f32() * random.next_f32();
            }
            *item = width_factor * width_factor;
        }
        width_factor_per_height
    }

    fn update_vertical_radius(
        config: &CarverConfig,
        random: &mut RandomGenerator,
        vertical_radius: f64,
        distance: f32,
        current_step: f32,
    ) -> f64 {
        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return vertical_radius;
        };
        let vertical_multiplier = 1.0 - (0.5 - current_step / distance).abs() * 2.0;
        let factor = canyon_config.shape.vertical_radius_default_factor
            + canyon_config.shape.vertical_radius_center_factor * vertical_multiplier;
        factor as f64 * vertical_radius * random.next_inbetween_f32(0.75, 1.0) as f64
    }

    fn should_skip(
        width_factor_per_height: &[f32],
        xd: f64,
        yd: f64,
        zd: f64,
        y: i32,
        min_gen_y: i32,
    ) -> bool {
        let y_index = (y - min_gen_y) as usize;
        if y_index == 0 {
            return true;
        }
        (xd * xd + zd * zd) * width_factor_per_height[y_index - 1] as f64 + (yd * yd) / 6.0 >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::carver::mask::CarvingMask;
    use pumpkin_data::carver::CANYON;
    use pumpkin_util::random::RandomGenerator;

    #[test]
    fn canyon_carve() {
        let chunk_pos = Vector2::new(0, 0);
        let carver_chunk_pos = Vector2::new(0, 0);
        let mut mask = CarvingMask::new(-64, 320);
        let mut random = RandomGenerator::Legacy(
            pumpkin_util::random::legacy_rand::LegacyRand::from_seed(12345),
        );

        CanyonCarver.carve(
            &CANYON,
            &mut mask,
            &mut random,
            &chunk_pos,
            &carver_chunk_pos,
            -64,
            384,
            false,
        );

        assert!(!mask.is_empty());
    }
}
