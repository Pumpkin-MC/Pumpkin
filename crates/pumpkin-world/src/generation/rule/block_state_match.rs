use pumpkin_data::BlockStateId;

pub struct BlockStateMatchRuleTest {
    pub block_state: BlockStateId,
}

impl BlockStateMatchRuleTest {
    #[must_use]
    pub const fn test(&self, state: BlockStateId) -> bool {
        self.block_state.as_u16() == state.as_u16()
    }
}
