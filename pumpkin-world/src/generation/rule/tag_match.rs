use pumpkin_data::{Block, BlockState, tag::Taggable};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TagMatchRuleTest {
    tag: String,
}

impl TagMatchRuleTest {
    pub fn test(&self, state: &BlockState) -> bool {
        Block::from_state_id(state.id)
            .is_tagged_with(&self.tag)
            .unwrap()
    }
}
