use pumpkin_data::Block;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

const SEA_LEVEL: i32 = 63; // TODO: use getSeaLevel() instead of hardcoding

pub struct BlueIceFeature {}

impl BlueIceFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if pos.0.y >= SEA_LEVEL - 1 {
            return false;
        }

        let block = GenerationCache::get_block_state(chunk, &pos.0).to_state();
        let block_below = GenerationCache::get_block_state(chunk, &pos.down().0).to_state();

        if block != Block::WATER.default_state && block_below != Block::WATER.default_state {
            return false;
        }

        let mut has_ice_neighbor = false;
        for neighbor_pos in [
            pos.up(),
            pos.down(),
            pos.north(),
            pos.south(),
            pos.east(),
            pos.west(),
        ] {
            if GenerationCache::get_block_state(chunk, &neighbor_pos.0).to_state()
                == Block::ICE.default_state
            {
                has_ice_neighbor = true;
                break;
            }
        }

        if !has_ice_neighbor {
            return false;
        }

        chunk.set_block_state(&pos.0, Block::BLUE_ICE.default_state);

        for _ in 0..200 {
            let offset_x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let offset_y = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let offset_z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let target_pos = pos.add(offset_x, offset_y, offset_z);

            if chunk.out_of_height(target_pos.0.y as i16) {
                continue;
            }

            let target_state = GenerationCache::get_block_state(chunk, &target_pos.0).to_state();
            if target_state.is_air()
                || target_state == Block::WATER.default_state
                || target_state == Block::ICE.default_state
                || target_state == Block::PACKED_ICE.default_state
            {
                let mut has_blue_ice_neighbor = false;
                for neighbor_pos in [
                    target_pos.up(),
                    target_pos.down(),
                    target_pos.north(),
                    target_pos.south(),
                    target_pos.east(),
                    target_pos.west(),
                ] {
                    if GenerationCache::get_block_state(chunk, &neighbor_pos.0).to_state()
                        == Block::BLUE_ICE.default_state
                    {
                        has_blue_ice_neighbor = true;
                        break;
                    }
                }

                if has_blue_ice_neighbor {
                    chunk.set_block_state(&target_pos.0, Block::BLUE_ICE.default_state);
                }
            }
        }

        true
    }
}
