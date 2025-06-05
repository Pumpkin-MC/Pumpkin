use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::block_state_provider::BlockStateProvider};

#[derive(Deserialize)]
pub struct AttachedToLogsTreeDecorator {
    probability: f32,
    block_provider: BlockStateProvider,
    directions: Vec<BlockDirection>,
}

impl AttachedToLogsTreeDecorator {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _root_positions: Vec<BlockPos>,
        log_positions: Vec<BlockPos>,
    ) {
        // TODO: shuffle
        for pos in log_positions {
            // TODO: random
            let pos = pos.offset(self.directions[0].to_offset());
            if random.next_f32() > self.probability
                || !chunk.get_block_state(&pos.0).to_state().is_air()
            {
                continue;
            }
            chunk.set_block_state(&pos.0, &self.block_provider.get(random, pos));
        }
    }
}
