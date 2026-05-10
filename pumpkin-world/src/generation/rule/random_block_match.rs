use pumpkin_data::Block;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use crate::block::RawBlockState;

pub struct RandomBlockMatchRuleTest {
    pub block: Block,
    pub probability: f32,
}

impl RandomBlockMatchRuleTest {
    pub fn test(&self, state: RawBlockState, random: &mut RandomGenerator) -> bool {
        state.to_block().name == self.block.name && random.next_f32() < self.probability
    }
}
