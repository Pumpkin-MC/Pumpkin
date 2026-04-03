use pumpkin_data::Block;

use crate::block::RawBlockState;

pub struct BlockMatchRuleTest {
    pub block: Block,
}

impl BlockMatchRuleTest {
    #[must_use]
    pub fn test(&self, state: RawBlockState) -> bool {
        state.to_block().name == self.block.name
    }
}
