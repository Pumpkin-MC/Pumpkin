use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, SmallDripleafLikeProperties, TallSeagrassLikeProperties,
};
use pumpkin_data::{Block, BlockState};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::block_state_provider::BlockStateProvider,
    world::{BlockAccessor, WorldPortalExt},
};

pub struct SimpleBlockFeature {
    pub to_place: BlockStateProvider,
    pub schedule_tick: Option<bool>,
}

impl SimpleBlockFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let state = self.to_place.get(random, pos, chunk, block_registry);
        let block = Block::from_state_id(state.id);
        let block_accessor: &dyn BlockAccessor = chunk;
        if !block_registry.can_place_at(block, state, block_accessor, &pos) {
            return false;
        }

        // Vanilla places both halves of DoublePlantBlocks here (DoublePlantBlock.placeAt)
        if TallSeagrassLikeProperties::handles_block_id(block.id) {
            let upper_pos = pos.up();
            if !chunk.is_air(&upper_pos.0) {
                return false;
            }
            chunk.set_block_state(&pos.0, state);
            let mut upper_props = TallSeagrassLikeProperties::from_state_id(state.id, block);
            upper_props.half = DoubleBlockHalf::Upper;
            chunk.set_block_state(
                &upper_pos.0,
                BlockState::from_id(upper_props.to_state_id(block)),
            );
            return true;
        }

        // ... and SmallDripleafBlocks (SmallDripleafBlock.placeAt)
        if SmallDripleafLikeProperties::handles_block_id(block.id) {
            let upper_pos = pos.up();
            if !chunk.is_air(&upper_pos.0) {
                return false;
            }
            chunk.set_block_state(&pos.0, state);
            let mut upper_props = SmallDripleafLikeProperties::from_state_id(state.id, block);
            upper_props.half = DoubleBlockHalf::Upper;
            chunk.set_block_state(
                &upper_pos.0,
                BlockState::from_id(upper_props.to_state_id(block)),
            );
            return true;
        }

        // TODO: check things..
        chunk.set_block_state(&pos.0, state);
        // TODO: schedule tick when needed
        true
    }
}
