use pumpkin_data::{Block, BlockState};

use crate::BlockStateId;

/// Instead of using a memory heavy normal BlockState This is used for internal representation in chunks to save memory
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawBlockState(pub BlockStateId);

impl RawBlockState {
    pub const AIR: RawBlockState = RawBlockState(0);

    #[inline]
    pub fn to_state(&self) -> &'static BlockState {
        BlockState::from_id(self.0)
    }

    #[inline]
    pub fn to_block(&self) -> &'static Block {
        Block::from_state_id(self.0)
    }

    #[inline]
    pub fn to_block_id(&self) -> u16 {
        Block::get_raw_id_from_state_id(self.0)
    }
}
