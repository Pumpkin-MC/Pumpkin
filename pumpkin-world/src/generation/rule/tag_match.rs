use pumpkin_data::tag::{self};

use crate::block::RawBlockState;

pub struct TagMatchRuleTest {
    pub tag: tag::Tag,
}

impl TagMatchRuleTest {
    #[must_use]
    pub fn test(&self, state: RawBlockState) -> bool {
        self.tag.1.contains(&state.to_block_id())
    }
}
