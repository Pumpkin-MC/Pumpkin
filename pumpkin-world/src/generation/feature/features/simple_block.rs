use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, generation::block_state_provider::BlockStateProvider};

#[derive(Deserialize)]
pub struct SimpleBlockFeature {
    to_place: BlockStateProvider,
    schedule_tick: Option<bool>,
}

impl SimpleBlockFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let state = self.to_place.get(random, pos);
        // TODO: check things..
        chunk.set_block_state(&pos.0, &state);
        // TODO: schedule tick when needed
        true
    }
}
