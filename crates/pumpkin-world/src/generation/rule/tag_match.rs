use pumpkin_data::{
    BlockStateId,
    tag::{self},
};

pub struct TagMatchRuleTest {
    pub tag: tag::Tag,
}

impl TagMatchRuleTest {
    #[must_use]
    pub fn test(&self, state: BlockStateId) -> bool {
        state.to_block_id().has_tag(self.tag)
    }
}
