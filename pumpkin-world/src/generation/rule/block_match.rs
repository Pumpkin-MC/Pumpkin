use pumpkin_data::{Block, BlockState};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BlockMatchRuleTest {
    // This should be a Block codec, so this is wrong
    block: String,
}

impl BlockMatchRuleTest {
    pub fn test(&self, state: &BlockState) -> bool {
        Block::from_state_id(state.id).name
            == self.block.strip_prefix("minecraft:").unwrap_or(&self.block)
    }
}
