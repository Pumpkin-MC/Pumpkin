use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, VineLikeProperties, get_state_by_state_id},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

#[derive(Deserialize)]
pub struct TrunkVineTreeDecorator;

impl TrunkVineTreeDecorator {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        log_positions: Vec<BlockPos>,
    ) {
        for pos in log_positions {
            if random.next_bounded_i32(3) > 0
                && chunk.is_air(&pos.offset(BlockDirection::West.to_offset()).0)
            {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.east = true;
                chunk.set_block_state(
                    &pos.offset(BlockDirection::West.to_offset()).0,
                    get_state_by_state_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0
                && chunk.is_air(&pos.offset(BlockDirection::East.to_offset()).0)
            {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.west = true;
                chunk.set_block_state(
                    &pos.offset(BlockDirection::West.to_offset()).0,
                    get_state_by_state_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0
                && chunk.is_air(&pos.offset(BlockDirection::North.to_offset()).0)
            {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.south = true;
                chunk.set_block_state(
                    &pos.offset(BlockDirection::West.to_offset()).0,
                    get_state_by_state_id(vine.to_state_id(&Block::VINE)),
                );
            }

            if random.next_bounded_i32(3) > 0
                && chunk.is_air(&pos.offset(BlockDirection::South.to_offset()).0)
            {
                let mut vine = VineLikeProperties::default(&Block::VINE);
                vine.north = true;
                chunk.set_block_state(
                    &pos.offset(BlockDirection::West.to_offset()).0,
                    get_state_by_state_id(vine.to_state_id(&Block::VINE)),
                );
            }
        }
    }
}
