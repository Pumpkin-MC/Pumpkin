use pumpkin_data::{Block, tag::Taggable};
use serde::Deserialize;

/// Rule tests are used in structure or features generation to check if a block state matches some condition.
#[derive(Deserialize)]
pub enum RuleTest {
    TagMatch(TagMatchTest),
    BlockMatch(BlockMatchRuleTest),
    // TODO: add more
}

pub struct AlwaysTrueRuleTest;

impl AlwaysTrueRuleTest {
    pub const fn test() -> bool {
        true
    }
}

#[derive(Deserialize)]
pub struct BlockMatchRuleTest {
    block: String,
}

impl BlockMatchRuleTest {
    pub fn test(&self, block: &'static Block) -> bool {
        let test_block = Block::from_registry_key(&self.block).expect("Failed to find block");
        test_block == block
    }
}

#[derive(Deserialize)]
pub struct TagMatchTest {
    tag: String,
}

impl TagMatchTest {
    pub fn test(&self, block: &'static Block) -> bool {
        block.is_tagged_with(&self.tag).unwrap()
    }
}
