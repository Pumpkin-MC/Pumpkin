use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct BlueIceFeature;

impl BlueIceFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if pos.0.y > chunk.get_sea_level() - 1 {
            return false;
        }

        let block = GenerationCache::get_block_state(chunk, &pos.0).to_state();
        let block_below = GenerationCache::get_block_state(chunk, &pos.down().0).to_state();

        if block != Block::WATER.default_state && block_below != Block::WATER.default_state {
            return false;
        }

        let mut found_packed_ice = false;
        for dir in BlockDirection::all() {
            if dir != BlockDirection::Down {
                let neighbor_pos = pos.offset(dir.to_offset());
                if GenerationCache::get_block_state(chunk, &neighbor_pos.0).to_block_id()
                    == Block::PACKED_ICE.id
                {
                    found_packed_ice = true;
                    break;
                }
            }
        }

        if !found_packed_ice {
            return false;
        }

        chunk.set_block_state(&pos.0, Block::BLUE_ICE.default_state);

        for _ in 0..200 {
            let y_off = random.next_bounded_i32(5) - random.next_bounded_i32(6);
            let mut xz_diff = 3;
            if y_off < 2 {
                xz_diff += y_off / 2;
            }

            if xz_diff >= 1 {
                let place_pos = pos.add(
                    random.next_bounded_i32(xz_diff) - random.next_bounded_i32(xz_diff),
                    y_off,
                    random.next_bounded_i32(xz_diff) - random.next_bounded_i32(xz_diff),
                );
                if chunk.out_of_height(place_pos.0.y as i16) {
                    continue;
                }

                let place_state = GenerationCache::get_block_state(chunk, &place_pos.0).to_state();
                if place_state.is_air()
                    || place_state == Block::WATER.default_state
                    || place_state == Block::PACKED_ICE.default_state
                    || place_state == Block::ICE.default_state
                {
                    for dir in BlockDirection::all() {
                        let rel_pos = place_pos.offset(dir.to_offset());
                        if GenerationCache::get_block_state(chunk, &rel_pos.0).to_block_id()
                            == Block::BLUE_ICE.id
                        {
                            chunk.set_block_state(&place_pos.0, Block::BLUE_ICE.default_state);
                            break;
                        }
                    }
                }
            }
        }

        true
    }
}
