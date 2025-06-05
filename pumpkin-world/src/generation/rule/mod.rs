use block_match::BlockMatchRuleTest;
use block_state_match::BlockStateMatchRuleTest;
use pumpkin_data::BlockState;
use pumpkin_util::random::RandomGenerator;
use random_block_match::RandomBlockMatchRuleTest;
use random_block_state_match::RandomBlockStateMatchRuleTest;
use serde::Deserialize;
use tag_match::TagMatchRuleTest;

mod block_match;
mod block_state_match;
mod random_block_match;
mod random_block_state_match;
mod tag_match;

#[derive(Deserialize)]
#[serde(tag = "predicate_type")]
pub enum RuleTest {
    #[serde(rename = "minecraft:always_true")]
    AlwaysTrue,
    #[serde(rename = "minecraft:block_match")]
    BlockMatch(BlockMatchRuleTest),
    #[serde(rename = "minecraft:blockstate_match")]
    BlockStateMatch(BlockStateMatchRuleTest),
    #[serde(rename = "minecraft:tag_match")]
    TagMatch(TagMatchRuleTest),
    #[serde(rename = "minecraft:random_block_match")]
    RandomBlockMatch(RandomBlockMatchRuleTest),
    #[serde(rename = "minecraft:random_blockstate_match")]
    RandomBlockStateMatch(RandomBlockStateMatchRuleTest),
}

impl RuleTest {
    pub fn test(&self, state: &BlockState, random: &mut RandomGenerator) -> bool {
        match self {
            RuleTest::AlwaysTrue => true,
            RuleTest::BlockMatch(rule) => rule.test(state),
            RuleTest::BlockStateMatch(rule) => rule.test(state),
            RuleTest::TagMatch(rule) => rule.test(state),
            RuleTest::RandomBlockMatch(rule) => rule.test(state, random),
            RuleTest::RandomBlockStateMatch(rule) => rule.test(state, random),
        }
    }
}
