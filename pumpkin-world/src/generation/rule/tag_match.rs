use pumpkin_data::{BlockState, block_properties::get_block_by_state_id, tag::Tagable};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TagMatchRuleTest {
    tag: String,
}

impl TagMatchRuleTest {
    pub fn test(&self, state: &BlockState) -> bool {
        get_block_by_state_id(state.id)
            .unwrap()
            .is_tagged_with(&self.tag)
            .unwrap()
    }
}
