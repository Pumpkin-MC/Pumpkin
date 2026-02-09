use serde::Deserialize;

use crate::block::RawBlockState;

#[derive(Deserialize)]
pub struct BlockMatchRuleTest {
    // This should be a Block codec, so this is wrong
    block: String,
}

impl BlockMatchRuleTest {
    pub fn test(&self, state: RawBlockState) -> bool {
        state.to_block().name == self.block.strip_prefix("minecraft:").unwrap_or(&self.block)
    }
}
