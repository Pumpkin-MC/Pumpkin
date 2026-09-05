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
        let CarverAdditionalConfig::Cave(ref cave_config) = config.additional else {
            return;
        };

        let min_y = run.chunk.generation_bottom_y() as i32;
        let height = run.chunk.generation_height();

        let max_distance = (4 * 2 - 1) << 4;

        let cave_count = cave_config.count.get(random);

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
                let room_vertical_radius_multiplier =
                    cave_config.room_vertical_radius_multiplier.get(random) as f64;
                let thickness = 1.0 + random.next_f32() * 6.0;
                Self::create_room(
                    run,
                    x as f64,
                    y,
                    z as f64,
                    thickness,
                    room_vertical_radius_multiplier,
                    config,
                    floor_level,
                );
                tunnels += random.next_bounded_i32(4);
            }

            for _ in 0..tunnels {
                let horizontal_rotation = random.next_f32() * PI * 2.0;
                let vertical_rotation = (random.next_f32() - 0.5) / 4.0;
                let mut thickness = cave_config.thickness.get(random);
                if cave_config.weird_thickness_bias && random.next_bounded_i32(10) == 0 {
                    thickness *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
                }
                let distance = max_distance - random.next_bounded_i32(max_distance / 4);
                let start_vertical_radius_multiplier =
                    cave_config.start_vertical_radius_multiplier.get(random) as f64;

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
                    start_vertical_radius_multiplier,
                    floor_level,
                    legacy_random_source,
                );
            }
        }
    }
}

impl CaveCarver {
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
                    horizontal_radius * horizontal_radius_multiplier,
                    vertical_radius * vertical_radius_multiplier,
                    floor_level,
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
    ) {
        let chunk_min_x = run.chunk.x << 4;
        let chunk_min_z = run.chunk.z << 4;

        if !Self::in_chunk_range(
            chunk_min_x,
            chunk_min_z,
            x,
            z,
            horizontal_radius,
            horizontal_radius,
        ) {
            return;
        }

        let min_y = 1.max((y - vertical_radius).floor() as i32 - 1);
        let max_y = (run.chunk.generation_height() as i32 - 1)
            .min((y + vertical_radius).floor() as i32 + 1);

        let x_index_min = ((x - horizontal_radius).floor() as i32 - chunk_min_x - 1).max(0);
        let x_index_max = ((x + horizontal_radius).floor() as i32 - chunk_min_x).min(15);

        let is_overworld = run.ctx.carver_aquifer.is_some();
        let protected_blocks_on_top = i32::from(!is_overworld);
        let max_y = max_y.min(
            (run.chunk.generation_bottom_y() as i32 + run.chunk.generation_height() as i32)
                - 1
                - protected_blocks_on_top,
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

    fn carve_block(
        run: &mut CarveRun,
        _config: &CarverConfig,
        x: i32,
        y: i32,
        z: i32,
        has_grass: &mut bool,
    ) -> bool {
        let state = run.chunk.get_block_state(&Vector3::new(x, y, z));
        let block = state.to_block();

        if block.id == pumpkin_data::Block::GRASS_BLOCK.id
            || block.id == pumpkin_data::Block::MYCELIUM.id
        {
            *has_grass = true;
        }

        let Some((state, should_schedule_fluid_update)) = overworld_carve_state(run, x, y, z)
        else {
            return false;
        };

        let overworld = run.ctx.carver_aquifer.is_some();

        place_carved_block(
            run,
            Vector3::new(x, y, z),
            state,
            should_schedule_fluid_update,
            *has_grass,
            overworld,
        );

        true
    }

    fn in_chunk_range(
        chunk_min_x: i32,
        chunk_min_z: i32,
        x: f64,
        z: f64,
        horizontal_radius_x: f64,
        horizontal_radius_z: f64,
    ) -> bool {
        x >= (chunk_min_x as f64 - 16.0 - horizontal_radius_x * 2.0)
            && z >= (chunk_min_z as f64 - 16.0 - horizontal_radius_z * 2.0)
            && x <= (chunk_min_x as f64 + 16.0 + 16.0 + horizontal_radius_x * 2.0)
            && z <= (chunk_min_z as f64 + 16.0 + 16.0 + horizontal_radius_z * 2.0)
    }
}

pub fn get_height(p: &HeightProvider, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
    match p {
        HeightProvider::Uniform(p) => {
            let min = p.min_inclusive.get_y(min_y as i16, height);
            let max = p.max_inclusive.get_y(min_y as i16, height);
            if min > max {
                min
            } else {
                random.next_inbetween_i32(min, max)
            }
        }
        HeightProvider::Trapezoid(p) => {
            let i = p.min_inclusive.get_y(min_y as i16, height);
            let j = p.max_inclusive.get_y(min_y as i16, height);
            if i > j {
                return i;
            }
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
            if max - min - inner < 0 {
                min
            } else {
                let upper_inclusive = random.next_inbetween_i32(min + inner, max);
                let biased_upper_inclusive = random.next_inbetween_i32(min, upper_inclusive - 1);
                random.next_inbetween_i32(min, biased_upper_inclusive - 1 + inner)
            }
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
            let expected = super::super::overworld_carve_state(run, 5, 20, 6)
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
        let carved = CaveCarver::carve_block(run, &CAVE, x, y, z, &mut has_grass);
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
        for y in -64..=63 {
            for x in 0..16 {
                for z in 0..16 {
                    let Some((state, should_schedule)) =
                        super::super::overworld_carve_state(run, x, y, z)
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
