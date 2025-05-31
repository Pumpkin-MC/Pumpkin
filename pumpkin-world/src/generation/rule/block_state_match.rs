use pumpkin_data::BlockState;
use serde::Deserialize;

use crate::block::BlockStateCodec;

#[derive(Deserialize)]
pub struct BlockStateMatchRuleTest {
    block_state: BlockStateCodec,
}

impl BlockStateMatchRuleTest {
    pub fn test(&self, state: &BlockState) -> bool {
        state.id == self.block_state.get_state().unwrap().id
    }
}
