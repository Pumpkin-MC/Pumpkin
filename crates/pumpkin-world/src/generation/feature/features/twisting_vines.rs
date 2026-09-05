use pumpkin_data::{Block, BlockState, block_properties::KelpLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct TwistingVinesFeature {
    pub spread_width: i32,
    pub spread_height: i32,
    pub max_height: i32,
}

impl TwistingVinesFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if Self::is_invalid_placement_location(chunk, pos) {
            return false;
        }

        let mut placed = false;

        for _ in 0..self.spread_width * self.spread_width {
            let offset_x = random.next_inbetween_i32(-self.spread_width, self.spread_width);
            let offset_y = random.next_inbetween_i32(-self.spread_height, self.spread_height);
            let offset_z = random.next_inbetween_i32(-self.spread_width, self.spread_width);

            let mut place_pos = pos.add(offset_x, offset_y, offset_z);

            if Self::find_first_air_block_above_ground(chunk, &mut place_pos)
                && !Self::is_invalid_placement_location(chunk, place_pos)
            {
                let mut vine_height = random.next_inbetween_i32(1, self.max_height);
                if random.next_bounded_i32(6) == 0 {
                    vine_height *= 2;
                }
                if random.next_bounded_i32(5) == 0 {
                    vine_height = 1;
                }

                Self::place_twisting_vines_column(chunk, random, place_pos, vine_height, 17, 25);
                placed = true;
            }
        }

        placed
    }

    fn find_first_air_block_above_ground<T: GenerationCache>(
        chunk: &T,
        place_pos: &mut BlockPos,
    ) -> bool {
        loop {
            *place_pos = place_pos.down();
            if place_pos.0.y <= chunk.bottom_y() as i32 {
                return false;
            }
            if !chunk.is_air(&place_pos.0) {
                break;
            }
        }

        *place_pos = place_pos.up();
        true
    }

    pub fn place_twisting_vines_column<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        mut place_pos: BlockPos,
        total_height: i32,
        min_age: i32,
        max_age: i32,
    ) {
        for height in 1..=total_height {
            if chunk.is_air(&place_pos.0) {
                if height == total_height || !chunk.is_air(&place_pos.up().0) {
                    let age = random.next_inbetween_i32(min_age, max_age);
                    let mut props = KelpLikeProperties::default(&Block::TWISTING_VINES);
                    props.age = age as u8;
                    let state_id = props.to_state_id(&Block::TWISTING_VINES);
                    chunk.set_block_state(&place_pos.0, BlockState::from_id(state_id));
                    break;
                }

                chunk.set_block_state(&place_pos.0, Block::TWISTING_VINES_PLANT.default_state);
            }

            place_pos = place_pos.up();
        }
    }

    fn is_invalid_placement_location<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        if !chunk.is_air(&pos.0) {
            return true;
        }

        let below = pos.down();
        let below_id = GenerationCache::get_block_state(chunk, &below.0).to_block_id();
        below_id != Block::NETHERRACK.id
            && below_id != Block::WARPED_NYLIUM.id
            && below_id != Block::WARPED_WART_BLOCK.id
    }
}
