use block_match::BlockMatchRuleTest;
use block_state_match::BlockStateMatchRuleTest;
use pumpkin_data::BlockStateId;
use pumpkin_util::random::RandomGenerator;
use random_block_match::RandomBlockMatchRuleTest;
use random_block_state_match::RandomBlockStateMatchRuleTest;
use tag_match::TagMatchRuleTest;

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
    // Added in 26.3
    AnyOf(Vec<Self>),
    AllOf(Vec<Self>),
    Not(Box<Self>),
    HeightMatch {
        min_inclusive: i32,
        max_inclusive: i32,
    },
}

impl RuleTest {
    pub fn test(&self, state: BlockStateId, y: i32, random: &mut RandomGenerator) -> bool {
        match self {
            Self::AlwaysTrue => true,
            Self::BlockMatch(rule) => rule.test(state),
            Self::BlockStateMatch(rule) => rule.test(state),
            Self::TagMatch(rule) => rule.test(state),
            Self::RandomBlockMatch(rule) => rule.test(state, random),
            Self::RandomBlockStateMatch(rule) => rule.test(state, random),
            Self::AnyOf(rules) => rules.iter().any(|rule| rule.test(state, y, random)),
            Self::AllOf(rules) => rules.iter().all(|rule| rule.test(state, y, random)),
            Self::Not(rule) => !rule.test(state, y, random),
            Self::HeightMatch {
                min_inclusive,
                max_inclusive,
            } => y >= *min_inclusive && y <= *max_inclusive,
        }
    }
}
