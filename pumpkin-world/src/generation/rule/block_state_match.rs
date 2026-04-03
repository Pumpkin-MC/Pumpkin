use pumpkin_data::BlockState;

use crate::block::RawBlockState;

pub struct BlockStateMatchRuleTest {
    pub block_state: BlockState,
}

impl BlockStateMatchRuleTest {
    #[must_use]
    pub fn test(&self, state: RawBlockState) -> bool {
        state.0 == self.block_state.id
    }
}
