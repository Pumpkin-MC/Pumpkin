use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, VineLikeProperties},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

pub struct TrunkVineTreeDecorator;

impl TrunkVineTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
    ) {
        for pos in log_positions {
            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.west().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.east = true;
                chunk.set_block_state(
                    &pos.west().0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.east().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.west = true;
                chunk.set_block_state(
                    &pos.east().0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.north().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.south = true;
                chunk.set_block_state(
                    &pos.north().0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.south().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.north = true;
                chunk.set_block_state(
                    &pos.south().0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
            }
        }
    }
}
