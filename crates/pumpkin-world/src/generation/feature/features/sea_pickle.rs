use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::SeaPickleLikeProperties};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct SeaPickleFeature {
    pub count: IntProvider,
}

impl SeaPickleFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed = 0;
        let count = self.count.get(random);

        for _ in 0..count {
            let x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let y = chunk.ocean_floor_height_exclusive(pos.0.x + x, pos.0.z + z);
            let pickle_pos = BlockPos::new(pos.0.x + x, y, pos.0.z + z);

            if GenerationCache::get_block_state(chunk, &pickle_pos.0).to_block_id() == Block::WATER
                && Self::can_survive(chunk, pickle_pos)
            {
                let mut props = SeaPickleLikeProperties::default(&Block::SEA_PICKLE);
                props.pickles = (random.next_bounded_i32(4) as u8) + 1;
                let state_id = props.to_state_id(&Block::SEA_PICKLE);
                chunk.set_block_state(&pickle_pos.0, BlockState::from_id(state_id));
                placed += 1;
            }
        }

        placed > 0
    }

    fn can_survive<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let below = pos.down();
        GenerationCache::get_block_state(chunk, &below.0)
            .to_state()
            .is_side_solid(BlockDirection::Up)
    }
}
