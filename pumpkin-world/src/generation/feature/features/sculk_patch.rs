use pumpkin_data::Block;
use pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct SculkPatchFeature {
    pub charge_count: i32,
    pub amount_per_charge: i32,
    pub spread_attempts: i32,
    pub growth_rounds: i32,
    pub spread_rounds: i32,
    pub extra_rare_growths: IntProvider,
    pub catalyst_chance: f32,
}

impl SculkPatchFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !self.can_spread_from(chunk, pos) {
            return false;
        }

        let mut spreader = SculkSpreader::new();
        let total_rounds = self.spread_rounds + self.growth_rounds;

        for round in 0..total_rounds {
            for _ in 0..self.charge_count {
                spreader.add_cursor(pos, self.amount_per_charge);
            }

            let spread_veins = round < self.spread_rounds;

            for _ in 0..self.spread_attempts {
                spreader.update_cursors(chunk, random, spread_veins);
            }

            spreader.clear();
        }

        let below = pos.down();
        if random.next_f32() <= self.catalyst_chance
            && GenerationCache::get_block_state(chunk, &below.0)
                .to_state()
                .is_solid()
        {
            chunk.set_block_state(&pos.0, Block::SCULK_CATALYST.default_state);
        }

        let extra_growths = self.extra_rare_growths.get(random);
        for _ in 0..extra_growths {
            let candidate = pos.offset(Vector3::new(
                random.next_bounded_i32(5) - 2,
                0,
                random.next_bounded_i32(5) - 2,
            ));
            let state = GenerationCache::get_block_state(chunk, &candidate.0).to_state();
            let below_candidate = candidate.down();
            let below_state =
                GenerationCache::get_block_state(chunk, &below_candidate.0).to_state();

            if state.is_air() && below_state.is_side_solid(pumpkin_data::BlockDirection::Up) {
                // TODO: set sculk shrieker with can_summon = true if possible
                chunk.set_block_state(&candidate.0, Block::SCULK_SHRIEKER.default_state);
            }
        }

        true
    }

    fn can_spread_from<T: GenerationCache>(&self, chunk: &T, pos: BlockPos) -> bool {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        let block_id = state.to_block_id();
        if is_sculk_behaviour(block_id) {
            true
        } else {
            if !chunk.is_air(&pos.0) && block_id != Block::WATER.id {
                return false;
            }
            // Check if any neighbor is solid
            for neighbor in [
                pos.offset(Vector3::new(1, 0, 0)),
                pos.offset(Vector3::new(-1, 0, 0)),
                pos.offset(Vector3::new(0, 1, 0)),
                pos.offset(Vector3::new(0, -1, 0)),
                pos.offset(Vector3::new(0, 0, 1)),
                pos.offset(Vector3::new(0, 0, -1)),
            ] {
                if GenerationCache::get_block_state(chunk, &neighbor.0)
                    .to_state()
                    .is_solid()
                {
                    return true;
                }
            }
            false
        }
    }
}

fn is_sculk_behaviour(block_id: u16) -> bool {
    block_id == Block::SCULK.id
        || block_id == Block::SCULK_VEIN.id
        || block_id == Block::SCULK_CATALYST.id
        || block_id == Block::SCULK_SHRIEKER.id
        || block_id == Block::SCULK_SENSOR.id
        || block_id == Block::CALIBRATED_SCULK_SENSOR.id
}

fn is_sculk_replaceable(block_id: u16) -> bool {
    MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN.1.contains(&block_id)
}

struct Cursor {
    pos: BlockPos,
    charge: i32,
}

struct SculkSpreader {
    cursors: Vec<Cursor>,
}

impl SculkSpreader {
    const fn new() -> Self {
        Self {
            cursors: Vec::new(),
        }
    }

    fn add_cursor(&mut self, pos: BlockPos, charge: i32) {
        self.cursors.push(Cursor { pos, charge });
    }

    fn clear(&mut self) {
        self.cursors.clear();
    }

    fn update_cursors<T: GenerationCache>(
        &mut self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        _spread_veins: bool,
    ) {
        let mut next_cursors = Vec::new();
        for mut cursor in self.cursors.drain(..) {
            if cursor.charge <= 0 {
                continue;
            }

            // In world gen, it picks one of 26 neighbors
            let dx = random.next_bounded_i32(3) - 1;
            let dy = random.next_bounded_i32(3) - 1;
            let dz = random.next_bounded_i32(3) - 1;
            let target_pos = cursor.pos.offset(Vector3::new(dx, dy, dz));

            let target_state = GenerationCache::get_block_state(chunk, &target_pos.0);
            let target_block_id = target_state.to_block_id();

            if is_sculk_replaceable(target_block_id) {
                chunk.set_block_state(&target_pos.0, Block::SCULK.default_state);
                cursor.pos = target_pos;
                cursor.charge -= 1;
            } else if target_block_id == Block::SCULK.id {
                cursor.pos = target_pos;
                cursor.charge -= 1;
            }

            if cursor.charge > 0 {
                next_cursors.push(cursor);
            }
        }
        self.cursors = next_cursors;
    }
}
