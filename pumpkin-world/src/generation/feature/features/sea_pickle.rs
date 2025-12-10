use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, EnumVariants, Integer1To4, SeaPickleLikeProperties},
};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SeaPickleFeature {
    count: IntProvider,
}

impl SeaPickleFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut times = 0;
        let count = self.count.get(random);
        for _ in 0..count {
            let x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
            let y = chunk.ocean_floor_height_exclusive(pos.0.x + x, pos.0.z + z);
            if GenerationCache::get_block_state(chunk, &pos.0).to_block() != &Block::WATER {
                continue;
            }
            let mut props = SeaPickleLikeProperties::default(&Block::SEA_PICKLE);
            props.pickles = Integer1To4::from_index(random.next_bounded_i32(4) as u16); // TODO: vanilla adds + 1, but this can crash
            let pos = BlockPos::new(pos.0.x + x, y, pos.0.z + z);
            chunk.set_block_state(
                &pos.0,
                BlockState::from_id(props.to_state_id(&Block::SEA_PICKLE)),
            );
            times += 1;
        }
        times > 0
    }
}
