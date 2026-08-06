use super::{CarveRun, Carver, overworld_carve_state, place_carved_block};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig, HeightProvider};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use std::f32::consts::PI;

pub struct CaveCarver;

impl Carver for CaveCarver {
    fn carve(
        &self,
        config: &CarverConfig,
        run: &mut CarveRun,
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

        let min_y = run.chunk.bottom_y() as i32;
        let height = run.chunk.height();

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
                    run,
                    x as f64,
                    y,
                    z as f64,
                    thickness,
                    y_scale,
                    config,
                    floor_level,
                    is_nether,
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
                    run,
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
                    is_nether,
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
        run: &mut CarveRun,
        x: f64,
        y: f64,
        z: f64,
        thickness: f32,
        y_scale: f64,
        config: &CarverConfig,
        floor_level: f64,
        is_nether: bool,
    ) {
        let horizontal_radius =
            1.5 + f64::from(pumpkin_util::math::sin(std::f32::consts::FRAC_PI_2) * thickness);
        let vertical_radius = horizontal_radius * y_scale;
        Self::carve_ellipsoid(
            run,
            config,
            x + 1.0,
            y,
            z,
            horizontal_radius,
            vertical_radius,
            floor_level,
            is_nether,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn create_tunnel(
        config: &CarverConfig,
        run: &mut CarveRun,
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
        is_nether: bool,
        legacy_random_source: bool,
    ) {
        let mut random = super::new_carver_random(tunnel_seed as u64, legacy_random_source);
        let split_point = random.next_bounded_i32(dist / 2) + dist / 4;
        let is_steep = random.next_bounded_i32(6) == 0;
        let mut y_rota = 0.0f32;
        let mut x_rota = 0.0f32;

        for current_step in step..dist {
            let progress_arg = PI * current_step as f32 / dist as f32;
            let horizontal_radius =
                1.5 + f64::from(pumpkin_util::math::sin(progress_arg) * thickness);
            let vertical_radius = horizontal_radius * y_scale;
            let cos_x = pumpkin_util::math::cos(vertical_rotation);
            x += f64::from(pumpkin_util::math::cos(horizontal_rotation) * cos_x);
            y += f64::from(pumpkin_util::math::sin(vertical_rotation));
            z += f64::from(pumpkin_util::math::sin(horizontal_rotation) * cos_x);

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
                    run,
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
                    is_nether,
                    legacy_random_source,
                );
                Self::create_tunnel(
                    config,
                    run,
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
                    is_nether,
                    legacy_random_source,
                );
                return;
            }

            if random.next_bounded_i32(4) != 0 {
                if !Self::can_reach(
                    run.chunk.x,
                    run.chunk.z,
                    x,
                    z,
                    current_step,
                    dist,
                    thickness,
                ) {
                    return;
                }

                Self::carve_ellipsoid(
                    run,
                    config,
                    x,
                    y,
                    z,
                    horizontal_radius as f64 * horizontal_radius_multiplier,
                    vertical_radius * vertical_radius_multiplier,
                    floor_level,
                    is_nether,
                );
            }
        }
    }

    #[must_use]
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
        run: &mut CarveRun,
        config: &CarverConfig,
        x: f64,
        y: f64,
        z: f64,
        horizontal_radius: f64,
        vertical_radius: f64,
        floor_level: f64,
        is_nether: bool,
    ) {
        let center_x = (run.chunk.x << 4) as f64 + 8.0;
        let center_z = (run.chunk.z << 4) as f64 + 8.0;
        let max_delta = 16.0 + horizontal_radius * 2.0;

        if (x - center_x).abs() > max_delta || (z - center_z).abs() > max_delta {
            return;
        }

        let chunk_min_x = run.chunk.x << 4;
        let chunk_min_z = run.chunk.z << 4;

        let x_index_min = ((x - horizontal_radius).floor() as i32 - chunk_min_x - 1).max(0);
        let x_index_max = ((x + horizontal_radius).floor() as i32 - chunk_min_x).min(15);

        let min_y = ((y - vertical_radius).floor() as i32 - 1).max(run.chunk.bottom_y() as i32 + 1);
        let protected_blocks_on_top = 7;
        let max_y = ((y + vertical_radius).floor() as i32 + 1).min(
            run.chunk.bottom_y() as i32 + run.chunk.height() as i32 - 1 - protected_blocks_on_top,
        );

        let z_index_min = ((z - horizontal_radius).floor() as i32 - chunk_min_z - 1).max(0);
        let z_index_max = ((z + horizontal_radius).floor() as i32 - chunk_min_z).min(15);

        for x_index in x_index_min..=x_index_max {
            let world_x = chunk_min_x + x_index;
            let xd = (world_x as f64 + 0.5 - x) / horizontal_radius;

            for z_index in z_index_min..=z_index_max {
                let world_z = chunk_min_z + z_index;
                let zd = (world_z as f64 + 0.5 - z) / horizontal_radius;

                if xd * xd + zd * zd < 1.0 {
                    let mut has_grass = false;

                    for world_y in (min_y + 1..=max_y).rev() {
                        let yd = (world_y as f64 - 0.5 - y) / vertical_radius;

                        if !Self::should_skip(xd, yd, zd, floor_level)
                            && !run.chunk.carving_mask.get(world_x, world_y, world_z)
                        {
                            run.chunk.carving_mask.set(world_x, world_y, world_z);
                            Self::carve_block(
                                run,
                                config,
                                world_x,
                                world_y,
                                world_z,
                                is_nether,
                                &mut has_grass,
                            );
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

    #[allow(clippy::too_many_arguments)]
    fn carve_block(
        run: &mut CarveRun,
        config: &CarverConfig,
        x: i32,
        y: i32,
        z: i32,
        is_nether: bool,
        has_grass: &mut bool,
    ) -> bool {
        let state = run.chunk.get_block_state(&Vector3::new(x, y, z));
        let block = state.to_block();

        if block.id == pumpkin_data::Block::GRASS_BLOCK.id
            || block.id == pumpkin_data::Block::MYCELIUM.id
        {
            *has_grass = true;
        }

        if !block.id.has_tag(config.replaceable) {
            return false;
        }

        let (state, should_schedule_fluid_update) = if is_nether {
            let state = if y <= run.chunk.bottom_y() as i32 + 31 {
                run.ids.lava
            } else {
                run.ids.cave_air
            };
            (state, false)
        } else {
            let Some(state) = overworld_carve_state(run, config, x, y, z) else {
                return false;
            };
            state
        };

        place_carved_block(
            run,
            Vector3::new(x, y, z),
            state,
            should_schedule_fluid_update,
            *has_grass,
            !is_nether,
        );

        true
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::carver::CAVE;
    use pumpkin_data::{Block, BlockStateId, dimension::Dimension};

    type Run<'a, 'b> = super::super::CarveRun<'a, 'b>;

    #[test]
    fn carves_at_world_y() {
        super::super::with_carve_run(Dimension::OVERWORLD, |run| {
            let expected = super::super::overworld_carve_state(run, &CAVE, 5, 20, 6)
                .expect("test position should carve")
                .0
                .id;
            assert_world_y(run, 5, 20, 6, expected);
            assert_world_y(run, 7, -58, 8, Block::LAVA.default_state.id);
        });
    }

    #[test]
    fn uses_aquifer_state() {
        super::super::with_carve_run(Dimension::OVERWORLD, |run| {
            let (x, y, z, expected_state) =
                find_aquifer_carve_state(run, |state, _| state.id != Block::AIR.default_state.id)
                    .expect("expected non-air aquifer carve state in test chunk");
            carve_at(run, x, y, z, Block::WATER.default_state, expected_state.id);
            assert_ne!(expected_state.id, Block::AIR.default_state.id);

            let (x, y, z, expected_state) =
                find_aquifer_carve_state(run, |state, schedule| state.is_liquid() && schedule)
                    .expect("expected scheduled aquifer fluid in test chunk");
            let old_tick_count = run.chunk.fluid_ticks.len();

            carve_at(run, x, y, z, Block::STONE.default_state, expected_state.id);
            assert_eq!(run.chunk.fluid_ticks.len(), old_tick_count + 1);
            let pos = run.chunk.fluid_ticks.last().unwrap().position.0;
            assert_eq!((pos.x, pos.y, pos.z), (x, y, z));
        });
    }

    fn assert_world_y(run: &mut Run, x: i32, y: i32, z: i32, expected: BlockStateId) {
        let old_wrong_y = y - run.chunk.bottom_y() as i32;
        let stone = Block::STONE.default_state;

        run.chunk.set_block_state(x, y, z, stone);
        run.chunk.set_block_state(x, old_wrong_y, z, stone);
        carve_at(run, x, y, z, stone, expected);
        assert_eq!(block_id(run, x, old_wrong_y, z), stone.id);
    }

    fn carve_at(
        run: &mut Run,
        x: i32,
        y: i32,
        z: i32,
        initial_state: &'static pumpkin_data::BlockState,
        expected: BlockStateId,
    ) {
        let mut has_grass = false;

        run.chunk.set_block_state(x, y, z, initial_state);
        let carved = CaveCarver::carve_block(run, &CAVE, x, y, z, false, &mut has_grass);
        assert!(carved);
        assert_eq!(block_id(run, x, y, z), expected);
    }

    fn block_id(run: &Run, x: i32, y: i32, z: i32) -> BlockStateId {
        run.chunk.get_block_state(&Vector3::new(x, y, z))
    }

    fn find_aquifer_carve_state(
        run: &mut Run,
        predicate: impl Fn(&'static pumpkin_data::BlockState, bool) -> bool,
    ) -> Option<(i32, i32, i32, &'static pumpkin_data::BlockState)> {
        let lava_y = CAVE
            .lava_level
            .get_y(run.chunk.bottom_y() as i16, run.chunk.height());

        for y in (lava_y + 1)..=63 {
            for x in 0..16 {
                for z in 0..16 {
                    let Some((state, should_schedule)) =
                        super::super::overworld_carve_state(run, &CAVE, x, y, z)
                    else {
                        continue;
                    };

                    if predicate(state, should_schedule) {
                        return Some((x, y, z, state));
                    }
                }
            }
        }

        None
    }
}
