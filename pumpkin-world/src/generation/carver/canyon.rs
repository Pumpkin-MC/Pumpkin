use super::Carver;
use super::cave::get_height;
use crate::ProtoChunk;
use pumpkin_data::block_state::BlockState;
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use std::f32::consts::PI;

pub struct CanyonCarver;

impl Carver for CanyonCarver {
    fn carve(
        &self,
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        legacy_random_source: bool,
    ) {
        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return;
        };

        let min_y = chunk.bottom_y() as i32;
        let height = chunk.height();

        let max_distance = (4 * 2 - 1) * 16;

        let x = (carver_chunk_pos.x << 4) + random.next_bounded_i32(16);
        let y = get_height(&config.y, random, min_y as i8, height);
        let z = (carver_chunk_pos.y << 4) + random.next_bounded_i32(16);

        let horizontal_rotation = random.next_f32() * PI * 2.0;
        let vertical_rotation = canyon_config.vertical_rotation.get(random);
        let y_scale = config.y_scale.get(random) as f64;
        let thickness = canyon_config.shape.thickness.get(random);
        let distance =
            (max_distance as f32 * canyon_config.shape.distance_factor.get(random)) as i32;

        self.do_carve(
            config,
            chunk,
            random.next_i64(),
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
        &self,
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
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
        let mut random = if legacy_random_source {
            RandomGenerator::Legacy(pumpkin_util::random::legacy_rand::LegacyRand::from_seed(
                tunnel_seed as u64,
            ))
        } else {
            RandomGenerator::Xoroshiro(pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(
                tunnel_seed as u64,
            ))
        };
        let width_factor_per_height =
            self.init_width_factors(chunk.height() as usize, config, &mut random);
        let mut y_rota = 0.0f32;
        let mut x_rota = 0.0f32;

        let CarverAdditionalConfig::Canyon(ref canyon_config) = config.additional else {
            return;
        };

        for current_step in step..distance {
            let mut horizontal_radius =
                (1.5 + (current_step as f32 * PI / distance as f32).sin() * thickness) as f64;
            let mut vertical_radius = horizontal_radius * y_scale;
            horizontal_radius *= canyon_config
                .shape
                .horizontal_radius_factor
                .get(&mut random) as f64;
            vertical_radius = self.update_vertical_radius(
                config,
                &mut random,
                vertical_radius,
                distance as f32,
                current_step as f32,
            );

            let xc = vertical_rotation.cos();
            let xs = vertical_rotation.sin();
            x += (horizontal_rotation.cos() * xc) as f64;
            y += xs as f64;
            z += (horizontal_rotation.sin() * xc) as f64;

            vertical_rotation *= 0.7;
            vertical_rotation += x_rota * 0.05;
            horizontal_rotation += y_rota * 0.05;
            x_rota *= 0.8;
            y_rota *= 0.5;
            x_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
            y_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;

            if random.next_bounded_i32(4) != 0 {
                if !self.can_reach(chunk.x, chunk.z, x, z, current_step, distance, thickness) {
                    return;
                }

                self.carve_ellipsoid(
                    chunk,
                    config,
                    x,
                    y,
                    z,
                    horizontal_radius,
                    vertical_radius,
                    &width_factor_per_height,
                );
            }
        }
    }

    fn init_width_factors(
        &self,
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
        &self,
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

    #[allow(clippy::too_many_arguments)]
    fn can_reach(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        x: f64,
        z: f64,
        step: i32,
        distance: i32,
        thickness: f32,
    ) -> bool {
        let chunk_middle_x = (chunk_x << 4) + 8;
        let chunk_middle_z = (chunk_z << 4) + 8;
        let dx = x - chunk_middle_x as f64;
        let dz = z - chunk_middle_z as f64;
        let remaining = (distance - step) as f64;
        let rr = (thickness + 2.0 + 16.0) as f64;
        dx * dx + dz * dz - remaining * remaining <= rr * rr
    }

    #[allow(clippy::too_many_arguments)]
    fn carve_ellipsoid(
        &self,
        chunk: &mut ProtoChunk,
        config: &CarverConfig,
        x: f64,
        y: f64,
        z: f64,
        horizontal_radius: f64,
        vertical_radius: f64,
        width_factor_per_height: &[f32],
    ) {
        let center_x = (chunk.x << 4) as f64 + 8.0;
        let center_z = (chunk.z << 4) as f64 + 8.0;
        let max_delta = 16.0 + horizontal_radius * 2.0;

        if (x - center_x).abs() > max_delta || (z - center_z).abs() > max_delta {
            return;
        }

        let chunk_min_x = chunk.x << 4;
        let chunk_min_z = chunk.z << 4;

        let min_x_index = ((x - horizontal_radius).floor() as i32 - chunk_min_x - 1).max(0);
        let max_x_index = ((x + horizontal_radius).floor() as i32 - chunk_min_x).min(15);

        let min_y = ((y - vertical_radius).floor() as i32 - 1).max(chunk.bottom_y() as i32 + 1);
        let protected_blocks_on_top = 7;
        let max_y = ((y + vertical_radius).floor() as i32 + 1)
            .min(chunk.bottom_y() as i32 + chunk.height() as i32 - 1 - protected_blocks_on_top);

        let min_z_index = ((z - horizontal_radius).floor() as i32 - chunk_min_z - 1).max(0);
        let max_z_index = ((z + horizontal_radius).floor() as i32 - chunk_min_z).min(15);

        for x_index in min_x_index..=max_x_index {
            let world_x = chunk_min_x + x_index;
            let xd = (world_x as f64 + 0.5 - x) / horizontal_radius;

            for z_index in min_z_index..=max_z_index {
                let world_z = chunk_min_z + z_index;
                let zd = (world_z as f64 + 0.5 - z) / horizontal_radius;

                if xd * xd + zd * zd < 1.0 {
                    for world_y in (min_y + 1..=max_y).rev() {
                        let yd = (world_y as f64 - 0.5 - y) / vertical_radius;

                        if !self.should_skip(
                            width_factor_per_height,
                            xd,
                            yd,
                            zd,
                            world_y,
                            chunk.bottom_y() as i32,
                        ) && !chunk.carving_mask.get(world_x, world_y, world_z)
                        {
                            chunk.carving_mask.set(world_x, world_y, world_z);
                            self.carve_block(chunk, config, world_x, world_y, world_z);
                        }
                    }
                }
            }
        }
    }

    fn should_skip(
        &self,
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
        (xd * xd + zd * zd) * width_factor_per_height[y_index - 1] as f64 + yd * yd / 6.0 >= 1.0
    }

    fn carve_block(
        &self,
        chunk: &mut ProtoChunk,
        config: &CarverConfig,
        x: i32,
        y: i32,
        z: i32,
    ) -> bool {
        let local_y = y - chunk.bottom_y() as i32;
        let state_id = chunk.get_block_state_raw(x & 15, local_y, z & 15);
        let block = pumpkin_data::Block::from_state_id(state_id);

        if block.id == pumpkin_data::Block::WATER.id || block.id == pumpkin_data::Block::LAVA.id {
            return false;
        }

        if config.replaceable.1.contains(&block.id) {
            let air = BlockState::from_id(pumpkin_data::Block::AIR.default_state.id);
            let lava = BlockState::from_id(pumpkin_data::Block::LAVA.default_state.id);

            let lava_y = config
                .lava_level
                .get_y(chunk.bottom_y() as i16, chunk.height());

            if y <= lava_y {
                chunk.set_block_state(x & 15, local_y, z & 15, lava);
            } else {
                chunk.set_block_state(x & 15, local_y, z & 15, air);
            }

            return true;
        }
        false
    }
}
