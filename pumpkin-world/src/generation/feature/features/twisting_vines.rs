use pumpkin_data::Block;
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
        _feature_name: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if self.is_invalid_location(chunk, &pos) {
            return false;
        }

        let mut placed = false;

        for _ in 0..self.spread_width * self.spread_width {
            let offset_x = random.next_bounded_i32(self.spread_width)
                - random.next_bounded_i32(self.spread_width);
            let offset_y = random.next_bounded_i32(self.spread_height)
                - random.next_bounded_i32(self.spread_height);
            let offset_z = random.next_bounded_i32(self.spread_width)
                - random.next_bounded_i32(self.spread_width);

            let mut mutable_pos = pos.add(offset_x, offset_y, offset_z);

            if self.find_target_y(chunk, &mut mutable_pos)
                && !self.is_invalid_location(chunk, &mutable_pos)
            {
                let mut height = random.next_bounded_i32(self.max_height) + 1;
                if random.next_bounded_i32(6) == 0 {
                    height *= 2;
                }
                if random.next_bounded_i32(10) == 0 {
                    height = 1;
                }

                self.generate_column(chunk, random, &mutable_pos, height);
                placed = true;
            }
        }

        placed
    }

    fn generate_column<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: &BlockPos,
        height: i32,
    ) {
        let mut current_pos = *pos;
        for i in 0..height {
            if !GenerationCache::get_block_state(chunk, &current_pos.0)
                .to_state()
                .is_air()
            {
                break;
            }

            if i == height - 1
                || !GenerationCache::get_block_state(chunk, &current_pos.up().0)
                    .to_state()
                    .is_air()
            {
                // Top part
                let _age = 17 + random.next_bounded_i32(25 - 17 + 1);
                // We should set the age property here, but Pumpkin's BlockState might not support it easily yet or we just use default
                // For now, let's just use the block.
                // TODO: Set age property
                chunk.set_block_state(&current_pos.0, Block::TWISTING_VINES.default_state);
                break;
            }
            chunk.set_block_state(&current_pos.0, Block::TWISTING_VINES_PLANT.default_state);
            current_pos = current_pos.up();
        }
    }

    fn is_invalid_location<T: GenerationCache>(&self, chunk: &T, pos: &BlockPos) -> bool {
        if !GenerationCache::get_block_state(chunk, &pos.0)
            .to_state()
            .is_air()
        {
            return true;
        }

        let block_below = GenerationCache::get_block_state(chunk, &pos.down().0).to_state();
        block_below != Block::WARPED_NYLIUM.default_state
            && block_below != Block::WARPED_WART_BLOCK.default_state
            && block_below != Block::TWISTING_VINES.default_state
            && block_below != Block::TWISTING_VINES_PLANT.default_state
    }

    fn find_target_y<T: GenerationCache>(&self, chunk: &T, pos: &mut BlockPos) -> bool {
        // Try to find a valid floor by looking down
        let mut current = *pos;
        for _ in 0..self.spread_height {
            if GenerationCache::get_block_state(chunk, &current.0)
                .to_state()
                .is_air()
            {
                let below = current.down();
                let block_below = GenerationCache::get_block_state(chunk, &below.0).to_state();
                if block_below == Block::WARPED_NYLIUM.default_state
                    || block_below == Block::WARPED_WART_BLOCK.default_state
                    || block_below == Block::TWISTING_VINES.default_state
                    || block_below == Block::TWISTING_VINES_PLANT.default_state
                {
                    *pos = current;
                    return true;
                }
            }
            current = current.down();
        }
        false
    }
}
