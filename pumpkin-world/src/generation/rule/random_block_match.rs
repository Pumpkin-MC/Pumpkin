use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use crate::block::RawBlockState;

#[derive(Deserialize)]
pub struct RandomBlockMatchRuleTest {
    // This should be a Block codec, so this is wrong
    pub block: String,
    pub probability: f32,
}

impl RandomBlockMatchRuleTest {
    pub fn test(&self, state: RawBlockState, random: &mut RandomGenerator) -> bool {
        state.to_block().name == self.block.strip_prefix("minecraft:").unwrap_or(&self.block)
            && random.next_f32() < self.probability
    }
}
