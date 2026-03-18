use pumpkin_data::Block;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct BasaltPillarFeature {}

impl BasaltPillarFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) || chunk.is_air(&pos.up().0) {
            return false;
        }

        let basalt = &Block::BASALT.default_state;

        let mut cur_pos = pos;
        let mut place_north = true;
        let mut place_south = true;
        let mut place_west = true;
        let mut place_east = true;

        while chunk.is_air(&cur_pos.0) {
            if chunk.out_of_height(cur_pos.0.y as i16) {
                return true;
            }
            chunk.set_block_state(&cur_pos.0, basalt);
            place_north = place_north && Self::place_hang_off(chunk, random, cur_pos.north());
            place_south = place_south && Self::place_hang_off(chunk, random, cur_pos.south());
            place_west = place_west && Self::place_hang_off(chunk, random, cur_pos.west());
            place_east = place_east && Self::place_hang_off(chunk, random, cur_pos.east());
            cur_pos = cur_pos.down();
        }

        cur_pos = cur_pos.up();
        Self::place_base_hang_off(chunk, random, cur_pos.north());
        Self::place_base_hang_off(chunk, random, cur_pos.south());
        Self::place_base_hang_off(chunk, random, cur_pos.west());
        Self::place_base_hang_off(chunk, random, cur_pos.east());
        cur_pos = cur_pos.down();

        for dx in -3i32..4 {
            for dz in -3i32..4 {
                let probability = dx.abs() * dz.abs();
                if random.next_bounded_i32(10) < 10 - probability {
                    let mut base_pos =
                        BlockPos::new(cur_pos.0.x + dx, cur_pos.0.y, cur_pos.0.z + dz);
                    let mut max_drop = 3i32;
                    while chunk.is_air(&base_pos.down().0) {
                        base_pos = base_pos.down();
                        max_drop -= 1;
                        if max_drop <= 0 {
                            break;
                        }
                    }
                    if !chunk.is_air(&base_pos.down().0) {
                        chunk.set_block_state(&base_pos.0, basalt);
                    }
                }
            }
        }

        true
    }

    fn place_base_hang_off<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        if random.next_bool() {
            chunk.set_block_state(&pos.0, Block::BASALT.default_state);
        }
    }

    fn place_hang_off<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if random.next_bounded_i32(10) != 0 {
            chunk.set_block_state(&pos.0, Block::BASALT.default_state);
            true
        } else {
            false
        }
    }
}
