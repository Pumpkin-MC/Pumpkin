use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use crate::block::{BlockStateCodec, RawBlockState};

#[derive(Deserialize)]
pub struct RandomBlockStateMatchRuleTest {
    block_state: BlockStateCodec,
    probability: f32,
}

impl RandomBlockStateMatchRuleTest {
    pub fn test(&self, state: RawBlockState, random: &mut RandomGenerator) -> bool {
        state.0 == self.block_state.get_state_id() && random.next_f32() < self.probability
    }
}
