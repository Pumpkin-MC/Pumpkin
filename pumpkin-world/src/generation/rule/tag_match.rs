use pumpkin_data::tag::{self};

use crate::block::RawBlockState;

pub struct TagMatchRuleTest {
    pub tag: tag::Tag,
}

impl TagMatchRuleTest {
    #[must_use]
    pub fn test(&self, state: RawBlockState) -> bool {
        state.to_block_id().has_tag(self.tag)
    }
}
