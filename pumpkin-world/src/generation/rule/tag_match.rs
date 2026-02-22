use pumpkin_data::tag::{RegistryKey, get_tag_ids};

use crate::block::RawBlockState;

pub struct TagMatchRuleTest {
    pub tag: String,
}

impl TagMatchRuleTest {
    pub fn test(&self, state: RawBlockState) -> bool {
        let values = get_tag_ids(RegistryKey::Block, &self.tag).unwrap();
        values.contains(&state.to_block_id())
    }
}
