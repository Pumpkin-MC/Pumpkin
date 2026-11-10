use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockState, block_properties::VineLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

pub struct TrunkVineTreeDecorator;

impl TrunkVineTreeDecorator {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
        decorations: &mut Vec<BlockPos>,
    ) {
        for pos in log_positions {
            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.west().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.east = true;
                let target = pos.west();
                chunk.set_block_state(
                    &target.0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
                decorations.push(target);
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.east().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.west = true;
                let target = pos.east();
                chunk.set_block_state(
                    &target.0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
                decorations.push(target);
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.north().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.south = true;
                let target = pos.north();
                chunk.set_block_state(
                    &target.0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
                decorations.push(target);
            }

            if random.next_bounded_i32(3) > 0 && chunk.is_air(&pos.south().0) {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.north = true;
                let target = pos.south();
                chunk.set_block_state(
                    &target.0,
                    BlockState::from_id(vine.to_state_id(&Block::VINE)),
                );
                decorations.push(target);
            }
        }
    }
}
