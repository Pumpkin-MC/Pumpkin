use serde::Deserialize;

use crate::block::{BlockStateCodec, RawBlockState};

#[derive(Deserialize)]
pub struct BlockStateMatchRuleTest {
    block_state: BlockStateCodec,
}

impl BlockStateMatchRuleTest {
    pub fn test(&self, state: &RawBlockState) -> bool {
        state.0 == self.block_state.get_state_id()
    }
}
