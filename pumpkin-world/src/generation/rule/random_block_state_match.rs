use pumpkin_data::BlockState;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use crate::block::BlockStateCodec;

#[derive(Deserialize)]
pub struct RandomBlockStateMatchRuleTest {
    block_state: BlockStateCodec,
    probability: f32,
}

impl RandomBlockStateMatchRuleTest {
    pub fn test(&self, state: &BlockState, random: &mut RandomGenerator) -> bool {
        state.id == self.block_state.get_state().unwrap().id && random.next_f32() < self.probability
    }
}
