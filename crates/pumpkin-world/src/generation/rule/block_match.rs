use pumpkin_data::{BlockId, BlockStateId};

pub struct BlockMatchRuleTest {
    pub block: BlockId,
}

impl BlockMatchRuleTest {
    #[must_use]
    pub const fn test(&self, state: BlockStateId) -> bool {
        state.to_block_id().as_u16() == self.block.as_u16()
    }
}
