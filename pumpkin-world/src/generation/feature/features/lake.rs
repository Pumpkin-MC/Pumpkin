use pumpkin_data::{Block, BlockState, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::{block_state_provider::BlockStateProvider, proto_chunk::GenerationCache};
use crate::world::WorldPortalExt;

pub struct LakeFeature {
    pub fluid: BlockStateProvider,
    pub barrier: BlockStateProvider,
}

impl LakeFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if pos.0.y <= chunk.bottom_y() as i32 + 4 {
            return false;
        }

        let origin = pos.add(-8, -4, -8);
        let mut grid = [false; 2048];
        let spots = random.next_bounded_i32(4) + 4;

        for _ in 0..spots {
            let xr = random.next_f64() * 6.0 + 3.0;
            let yr = random.next_f64() * 4.0 + 2.0;
            let zr = random.next_f64() * 6.0 + 3.0;
            let xp = random.next_f64() * (16.0 - xr - 2.0) + 1.0 + xr / 2.0;
            let yp = random.next_f64() * (8.0 - yr - 4.0) + 2.0 + yr / 2.0;
            let zp = random.next_f64() * (16.0 - zr - 2.0) + 1.0 + zr / 2.0;

            for xx in 1..15 {
                for zz in 1..15 {
                    for yy in 1..7 {
                        let xd = (xx as f64 - xp) / (xr / 2.0);
                        let yd = (yy as f64 - yp) / (yr / 2.0);
                        let zd = (zz as f64 - zp) / (zr / 2.0);
                        let d = xd * xd + yd * yd + zd * zd;
                        if d < 1.0 {
                            grid[(xx * 16 + zz) * 8 + yy] = true;
                        }
                    }
                }
            }
        }

        let fluid_state = self
            .fluid
            .get_with_context(block_registry, chunk, random, origin);

        for xx in 0..16 {
            for zz in 0..16 {
                for yyx in 0..8 {
                    let check = !grid[(xx * 16 + zz) * 8 + yyx]
                        && (xx < 15 && grid[((xx + 1) * 16 + zz) * 8 + yyx]
                            || xx > 0 && grid[((xx - 1) * 16 + zz) * 8 + yyx]
                            || zz < 15 && grid[(xx * 16 + zz + 1) * 8 + yyx]
                            || zz > 0 && grid[(xx * 16 + (zz - 1)) * 8 + yyx]
                            || yyx < 7 && grid[(xx * 16 + zz) * 8 + yyx + 1]
                            || yyx > 0 && grid[(xx * 16 + zz) * 8 + (yyx - 1)]);

                    if check {
                        let check_pos = origin.add(xx as i32, yyx as i32, zz as i32);
                        let block_state =
                            GenerationCache::get_block_state(chunk, &check_pos.0).to_state();

                        if yyx >= 4 && block_state.is_liquid() {
                            return false;
                        }

                        if yyx < 4 && !block_state.is_solid() && block_state != fluid_state {
                            return false;
                        }
                    }
                }
            }
        }

        let air_state = Block::CAVE_AIR.default_state;

        for xx in 0..16 {
            for zz in 0..16 {
                for yyx in 0..8 {
                    if grid[(xx * 16 + zz) * 8 + yyx] {
                        let place_pos = origin.add(xx as i32, yyx as i32, zz as i32);
                        let current_state =
                            GenerationCache::get_block_state(chunk, &place_pos.0).to_state();
                        if self.can_replace_block(current_state) {
                            let place_air = yyx >= 4;
                            chunk.set_block_state(
                                &place_pos.0,
                                if place_air { air_state } else { fluid_state },
                            );
                        }
                    }
                }
            }
        }

        let barrier_state = self
            .barrier
            .get_with_context(block_registry, chunk, random, origin);
        if !barrier_state.is_air() {
            for xx in 0..16 {
                for zz in 0..16 {
                    for yyx in 0..8 {
                        let check = !grid[(xx * 16 + zz) * 8 + yyx]
                            && (xx < 15 && grid[((xx + 1) * 16 + zz) * 8 + yyx]
                                || xx > 0 && grid[((xx - 1) * 16 + zz) * 8 + yyx]
                                || zz < 15 && grid[(xx * 16 + zz + 1) * 8 + yyx]
                                || zz > 0 && grid[(xx * 16 + (zz - 1)) * 8 + yyx]
                                || yyx < 7 && grid[(xx * 16 + zz) * 8 + yyx + 1]
                                || yyx > 0 && grid[(xx * 16 + zz) * 8 + (yyx - 1)]);

                        if check && (yyx < 4 || random.next_bounded_i32(2) != 0) {
                            let barrier_pos = origin.add(xx as i32, yyx as i32, zz as i32);
                            let block_state =
                                GenerationCache::get_block_state(chunk, &barrier_pos.0).to_state();
                            if block_state.is_solid()
                                && !tag::Block::MINECRAFT_LAVA_POOL_STONE_CANNOT_REPLACE
                                    .1
                                    .contains(&block_state.id)
                            {
                                chunk.set_block_state(&barrier_pos.0, barrier_state);
                            }
                        }
                    }
                }
            }
        }

        // Freeze top layer
        if Block::from_state_id(fluid_state.id).id == Block::WATER.id {
            for xx in 0..16 {
                for zz in 0..16 {
                    let freeze_pos = origin.add(xx, 4, zz);
                    let biome = chunk.get_biome_for_terrain_gen(
                        freeze_pos.0.x,
                        freeze_pos.0.y,
                        freeze_pos.0.z,
                    );
                    if biome.weather.compute_temperature(
                        freeze_pos.0.x as f64,
                        freeze_pos.0.y,
                        freeze_pos.0.z as f64,
                        63,
                    ) < 0.15
                    {
                        let current_state =
                            GenerationCache::get_block_state(chunk, &freeze_pos.0).to_state();
                        if self.can_replace_block(current_state) {
                            chunk.set_block_state(&freeze_pos.0, Block::ICE.default_state);
                        }
                    }
                }
            }
        }

        true
    }

    fn can_replace_block(&self, state: &BlockState) -> bool {
        !tag::Block::MINECRAFT_FEATURES_CANNOT_REPLACE
            .1
            .contains(&state.id)
    }
}
