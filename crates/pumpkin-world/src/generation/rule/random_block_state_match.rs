use pumpkin_data::BlockStateId;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

pub struct RandomBlockStateMatchRuleTest {
    pub block_state: BlockStateId,
    pub probability: f32,
}

impl RandomBlockStateMatchRuleTest {
    pub fn test(&self, state: BlockStateId, random: &mut RandomGenerator) -> bool {
        self.block_state.as_u16() == state.as_u16() && random.next_f32() < self.probability
    }
}
