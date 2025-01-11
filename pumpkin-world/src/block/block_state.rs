use std::collections::HashMap;

use super::block_registry::{get_block, get_block_by_id, get_state_by_state_id};

#[derive(Clone, Copy, Debug, Eq)]
pub struct BlockState {
    pub state_id: u16,
    pub block_id: u16,
}

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.state_id == other.state_id
    }
}

impl BlockState {
    pub const AIR: BlockState = BlockState {
        state_id: 0,
        block_id: 0,
    };

    /// Get a Block from the Vanilla Block registry at Runtime
    pub fn new(registry_id: &str) -> Option<Self> {
        let block = get_block(registry_id);
        block.map(|block| Self {
            state_id: block.default_state_id,
            block_id: block.id,
        })
    }

    pub fn from_block(block_id: u16) -> Option<Self> {
        let block = get_block_by_id(block_id);

        block.map(|block| Self {
            state_id: block.default_state_id,
            block_id: block.id,
        })
    }

    pub fn from_block_and_props(
        _block_id: u16,
        _properties: HashMap<String, String>,
    ) -> Option<Self> {
        todo!()
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        get_state_by_state_id(self.state_id).unwrap().air
    }

    #[inline]
    pub fn of_block(&self, block_id: u16) -> bool {
        self.block_id == block_id
    }

    #[inline]
    pub const fn as_u32(&self) -> u32 {
        (self.block_id as u32) | (self.state_id as u32)
    }

    #[inline]
    pub const fn as_i32(&self) -> i32 {
        (self.block_id as i32) | (self.state_id as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::BlockState;

    #[test]
    fn not_existing() {
        let result = BlockState::new("this_block_does_not_exist");
        assert!(result.is_none());
    }

    #[test]
    fn does_exist() {
        let result = BlockState::new("dirt");
        assert!(result.is_some());
    }
}
