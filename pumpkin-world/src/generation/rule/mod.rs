use block_match::BlockMatchRuleTest;
use block_state_match::BlockStateMatchRuleTest;
use pumpkin_util::random::RandomGenerator;
use random_block_match::RandomBlockMatchRuleTest;
use random_block_state_match::RandomBlockStateMatchRuleTest;
use tag_match::TagMatchRuleTest;

use crate::block::RawBlockState;

pub mod block_match;
pub mod block_state_match;
pub mod random_block_match;
pub mod random_block_state_match;
pub mod tag_match;

pub enum RuleTest {
    AlwaysTrue,
    BlockMatch(BlockMatchRuleTest),
    BlockStateMatch(BlockStateMatchRuleTest),
    TagMatch(TagMatchRuleTest),
    RandomBlockMatch(RandomBlockMatchRuleTest),
    RandomBlockStateMatch(RandomBlockStateMatchRuleTest),
}

impl RuleTest {
    pub fn test(&self, state: RawBlockState, random: &mut RandomGenerator) -> bool {
        match self {
            Self::AlwaysTrue => true,
            Self::BlockMatch(rule) => rule.test(state),
            Self::BlockStateMatch(rule) => rule.test(state),
            Self::TagMatch(rule) => rule.test(state),
            Self::RandomBlockMatch(rule) => rule.test(state, random),
            Self::RandomBlockStateMatch(rule) => rule.test(state, random),
        }
    }
}
