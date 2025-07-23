use pumpkin_data::{BlockDirection, block_properties::get_block_by_state_id};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::block_state_provider::BlockStateProvider,
    world::{BlockAccessor, BlockRegistryExt},
};

#[derive(Deserialize)]
pub struct SimpleBlockFeature {
    to_place: BlockStateProvider,
    schedule_tick: Option<bool>,
}

impl SimpleBlockFeature {
    pub fn generate(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let state = self.to_place.get(random, pos);
        let block = get_block_by_state_id(state.id);
        let block_accessor: &dyn BlockAccessor = chunk;
        if !block_registry.can_place_at(block, block_accessor, &pos, BlockDirection::Up) {
            return false;
        }

        // TODO: check things..
        chunk.set_block_state(&pos.0, state);
        // TODO: schedule tick when needed
        true
    }
}
