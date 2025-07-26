use pumpkin_data::{Block, BlockState};

use crate::BlockStateId;

/// Instead of using a memory heavy normal BlockState This is used for internal representation in chunks to save memory
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawBlockState(pub BlockStateId);

impl RawBlockState {
    pub const AIR: RawBlockState = RawBlockState(0);

    /// Get a Block from the Vanilla Block registry at Runtime
    pub fn new(registry_id: &str) -> Option<Self> {
        let block = Block::from_name(registry_id);
        block.map(|block| Self(block.default_state.id))
    }

    #[inline]
    pub fn to_state(&self) -> &'static BlockState {
        BlockState::from_id(self.0)
    }

    #[inline]
    pub fn to_block(&self) -> &'static Block {
        Block::from_state_id(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::RawBlockState;

    #[test]
    fn not_existing() {
        let result = RawBlockState::new("this_block_does_not_exist");
        assert!(result.is_none());
    }

    #[test]
    fn does_exist() {
        let result = RawBlockState::new("dirt");
        assert!(result.is_some());
    }
}
