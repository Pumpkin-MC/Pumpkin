use super::Carver;
use crate::ProtoChunk;
use crate::generation::noise::aquifer_sampler::FluidLevelSamplerImpl;
use pumpkin_data::block_state::BlockState;
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig, HeightProvider};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use std::f32::consts::PI;

pub struct CaveCarver;

impl Carver for CaveCarver {
    fn carve(
        &self,
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        legacy_random_source: bool,
    ) {
        let (is_nether, cave_config) = match config.additional {
            CarverAdditionalConfig::Cave(ref c) => (false, c),
            CarverAdditionalConfig::NetherCave(ref c) => (true, c),
            CarverAdditionalConfig::Canyon(_) => return,
        };

        let min_y = chunk.bottom_y() as i32;
        let height = chunk.height();

        let max_distance = (4 * 2 - 1) << 4;

        let bound = if is_nether { 10 } else { 15 };
        let c1 = random.next_bounded_i32(bound);
        let c2 = random.next_bounded_i32(c1 + 1);
        let cave_count = random.next_bounded_i32(c2 + 1);

        for _ in 0..cave_count {
            let x = (carver_chunk_pos.x << 4) + random.next_bounded_i32(16);
            let y = get_height(&config.y, random, min_y as i8, height) as f64;
            let z = (carver_chunk_pos.y << 4) + random.next_bounded_i32(16);

            let horizontal_radius_multiplier =
                cave_config.horizontal_radius_multiplier.get(random) as f64;
            let vertical_radius_multiplier =
                cave_config.vertical_radius_multiplier.get(random) as f64;
            let floor_level = cave_config.floor_level.get(random) as f64;

            let mut tunnels = 1;
            if random.next_bounded_i32(4) == 0 {
                let y_scale = config.y_scale.get(random) as f64;
                let thickness = 1.0 + random.next_f32() * 6.0;
                Self::create_room(
                    chunk,
                    x as f64,
                    y,
                    z as f64,
                    thickness,
                    y_scale,
                    config,
                    floor_level,
                );
                tunnels += random.next_bounded_i32(4);
            }

            for _ in 0..tunnels {
                let horizontal_rotation = random.next_f32() * PI * 2.0;
                let vertical_rotation = (random.next_f32() - 0.5) / 4.0;
                let thickness = Self::get_thickness(random, is_nether);
                let distance = max_distance - random.next_bounded_i32(max_distance / 4);

                Self::create_tunnel(
                    config,
                    chunk,
                    random.next_i64(),
                    x as f64,
                    y,
                    z as f64,
                    horizontal_radius_multiplier,
                    vertical_radius_multiplier,
                    thickness,
                    horizontal_rotation,
                    vertical_rotation,
                    0,
                    distance,
                    if is_nether { 5.0 } else { 1.0 }, // this.getYScale()
                    floor_level,
                    legacy_random_source,
                );
            }
        }
    }
}

impl CaveCarver {
    fn get_thickness(random: &mut RandomGenerator, is_nether: bool) -> f32 {
        if is_nether {
            (random.next_f32() * 2.0 + random.next_f32()) * 2.0
        } else {
            let mut thickness = random.next_f32() * 2.0 + random.next_f32();
            if random.next_bounded_i32(10) == 0 {
                thickness *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
            }
            thickness
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_room(
        chunk: &mut ProtoChunk,
        x: f64,
        y: f64,
        z: f64,
        thickness: f32,
        y_scale: f64,
        config: &CarverConfig,
        floor_level: f64,
    ) {
        let horizontal_radius = 1.5 + (PI / 2.0).sin() * thickness;
        let vertical_radius = horizontal_radius as f64 * y_scale;
        Self::carve_ellipsoid(
            chunk,
            config,
            x + 1.0,
            y,
            z,
            horizontal_radius as f64,
            vertical_radius,
            floor_level,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn create_tunnel(
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
        tunnel_seed: i64,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        horizontal_radius_multiplier: f64,
        vertical_radius_multiplier: f64,
        thickness: f32,
        mut horizontal_rotation: f32,
        mut vertical_rotation: f32,
        step: i32,
        dist: i32,
        y_scale: f64,
        floor_level: f64,
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
        let split_point = random.next_bounded_i32(dist / 2) + dist / 4;
        let is_steep = random.next_bounded_i32(6) == 0;
        let mut y_rota = 0.0f32;
        let mut x_rota = 0.0f32;

        for current_step in step..dist {
            let horizontal_radius =
                1.5 + (PI * current_step as f32 / dist as f32).sin() * thickness;
            let vertical_radius = horizontal_radius as f64 * y_scale;
            let cos_x = vertical_rotation.cos();
            x += (horizontal_rotation.cos() * cos_x) as f64;
            y += vertical_rotation.sin() as f64;
            z += (horizontal_rotation.sin() * cos_x) as f64;

            vertical_rotation *= if is_steep { 0.92 } else { 0.7 };
            vertical_rotation += x_rota * 0.1;
            horizontal_rotation += y_rota * 0.1;
            x_rota *= 0.9;
            y_rota *= 0.75;
            x_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
            y_rota += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;

            if current_step == split_point && thickness > 1.0 {
                Self::create_tunnel(
                    config,
                    chunk,
                    random.next_i64(),
                    x,
                    y,
                    z,
                    horizontal_radius_multiplier,
                    vertical_radius_multiplier,
                    random.next_f32() * 0.5 + 0.5,
                    horizontal_rotation - (PI / 2.0),
                    vertical_rotation / 3.0,
                    current_step,
                    dist,
                    1.0,
                    floor_level,
                    legacy_random_source,
                );
                Self::create_tunnel(
                    config,
                    chunk,
                    random.next_i64(),
                    x,
                    y,
                    z,
                    horizontal_radius_multiplier,
                    vertical_radius_multiplier,
                    random.next_f32() * 0.5 + 0.5,
                    horizontal_rotation + (PI / 2.0),
                    vertical_rotation / 3.0,
                    current_step,
                    dist,
                    1.0,
                    floor_level,
                    legacy_random_source,
                );
                return;
            }

            if random.next_bounded_i32(4) != 0 {
                if !Self::can_reach(chunk.x, chunk.z, x, z, current_step, dist, thickness) {
                    return;
                }

                Self::carve_ellipsoid(
                    chunk,
                    config,
                    x,
                    y,
                    z,
                    horizontal_radius as f64 * horizontal_radius_multiplier,
                    vertical_radius * vertical_radius_multiplier,
                    floor_level,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn can_reach(
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
        chunk: &mut ProtoChunk,
        config: &CarverConfig,
        x: f64,
        y: f64,
        z: f64,
        horizontal_radius: f64,
        vertical_radius: f64,
        floor_level: f64,
    ) {
        let center_x = (chunk.x << 4) as f64 + 8.0;
        let center_z = (chunk.z << 4) as f64 + 8.0;
        let max_delta = 16.0 + horizontal_radius * 2.0;

        if (x - center_x).abs() > max_delta || (z - center_z).abs() > max_delta {
            return;
        }

        let chunk_min_x = chunk.x << 4;
        let chunk_min_z = chunk.z << 4;

        let min_x_idx = ((x - horizontal_radius).floor() as i32 - chunk_min_x - 1).max(0);
        let max_x_idx = ((x + horizontal_radius).floor() as i32 - chunk_min_x).min(15);

        let min_y = ((y - vertical_radius).floor() as i32 - 1).max(chunk.bottom_y() as i32 + 1);
        let protected_blocks_on_top = 7;
        let max_y = ((y + vertical_radius).floor() as i32 + 1)
            .min(chunk.bottom_y() as i32 + chunk.height() as i32 - 1 - protected_blocks_on_top);

        let min_z_idx = ((z - horizontal_radius).floor() as i32 - chunk_min_z - 1).max(0);
        let max_z_idx = ((z + horizontal_radius).floor() as i32 - chunk_min_z).min(15);

        for x_index in min_x_idx..=max_x_idx {
            let world_x = chunk_min_x + x_index;
            let xd = (world_x as f64 + 0.5 - x) / horizontal_radius;

            for z_index in min_z_idx..=max_z_idx {
                let world_z = chunk_min_z + z_index;
                let zd = (world_z as f64 + 0.5 - z) / horizontal_radius;

                if xd * xd + zd * zd < 1.0 {
                    let mut has_grass = false;
                    for world_y in (min_y + 1..=max_y).rev() {
                        let yd = (world_y as f64 - 0.5 - y) / vertical_radius;

                        if !Self::should_skip(xd, yd, zd, floor_level)
                            && !chunk.carving_mask.get(world_x, world_y, world_z)
                        {
                            chunk.carving_mask.set(world_x, world_y, world_z);
                            Self::carve_block(chunk, config, world_x, world_y, world_z, &mut has_grass);
                        }
                    }
                }
            }
        }
    }

    fn should_skip(xd: f64, yd: f64, zd: f64, floor_level: f64) -> bool {
        if yd <= floor_level {
            true
        } else {
            xd * xd + yd * yd + zd * zd >= 1.0
        }
    }

    fn carve_block(chunk: &mut ProtoChunk, config: &CarverConfig, x: i32, y: i32, z: i32, has_grass: &mut bool) -> bool {
        let local_y = y - chunk.bottom_y() as i32;
        let state_id = chunk.get_block_state_raw(x & 15, local_y, z & 15);
        let block = pumpkin_data::Block::from_state_id(state_id);

        if block.id == pumpkin_data::Block::GRASS_BLOCK.id
            || block.id == pumpkin_data::Block::MYCELIUM.id
        {
            *has_grass = true;
        }

        // Only carve if it's replaceable
        if config.replaceable.1.contains(&block.id) {
            let fluid_level = chunk.fluid_level_sampler.get_fluid_level(x, y, z);
            let replacement_block = fluid_level.get_block(y);
            let replacement_state = BlockState::from_id(replacement_block.default_state.id);

            chunk.set_block_state(x & 15, local_y, z & 15, replacement_state);

            if *has_grass {
                let below_state_id = chunk.get_block_state_raw(x & 15, local_y - 1, z & 15);
                let below_block = pumpkin_data::Block::from_state_id(below_state_id);

                if below_block.id == pumpkin_data::Block::DIRT.id {
                    let top_material =
                        BlockState::from_id(pumpkin_data::Block::GRASS_BLOCK.default_state.id);
                    chunk.set_block_state(x & 15, local_y - 1, z & 15, top_material);
                }
            }

            return true;
        }
        false
    }
}

pub fn get_height(p: &HeightProvider, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
    match p {
        HeightProvider::Uniform(p) => {
            let min = p.min_inclusive.get_y(min_y as i16, height);
            let max = p.max_inclusive.get_y(min_y as i16, height);
            random.next_inbetween_i32(min, max)
        }
        HeightProvider::Trapezoid(p) => {
            let i = p.min_inclusive.get_y(min_y as i16, height);
            let j = p.max_inclusive.get_y(min_y as i16, height);
            let plateau = p.plateau.unwrap_or(0);
            let k = j - i;
            if plateau >= k {
                random.next_inbetween_i32(i, j)
            } else {
                let l = (k - plateau) / 2;
                let m = k - l;
                i + random.next_inbetween_i32(0, m) + random.next_inbetween_i32(0, l)
            }
        }
        HeightProvider::VeryBiasedToBottom(p) => {
            let min = p.min_inclusive.get_y(min_y as i16, height);
            let max = p.max_inclusive.get_y(min_y as i16, height);
            let inner = p.inner.map_or(1, std::num::NonZero::get) as i32;
            let min_rnd = random.next_inbetween_i32(min + inner, max);
            let max_rnd = random.next_inbetween_i32(min, min_rnd - 1);
            random.next_inbetween_i32(min, max_rnd - 1 + inner)
        }
    }
}
