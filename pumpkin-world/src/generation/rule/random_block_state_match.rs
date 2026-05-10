use pumpkin_data::BlockState;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use crate::block::RawBlockState;

pub struct RandomBlockStateMatchRuleTest {
    pub block_state: BlockState,
    pub probability: f32,
}

impl RandomBlockStateMatchRuleTest {
    pub fn test(&self, state: RawBlockState, random: &mut RandomGenerator) -> bool {
        state.0 == self.block_state.id && random.next_f32() < self.probability
    }
}
