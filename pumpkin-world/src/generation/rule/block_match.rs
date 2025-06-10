use pumpkin_data::{BlockState, block_properties::get_block_by_state_id};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BlockMatchRuleTest {
    // This should be a Block codec, so this is wrong
    block: String,
}

impl BlockMatchRuleTest {
    pub fn test(&self, state: &BlockState) -> bool {
        get_block_by_state_id(state.id).unwrap().name == self.block
    }
}
