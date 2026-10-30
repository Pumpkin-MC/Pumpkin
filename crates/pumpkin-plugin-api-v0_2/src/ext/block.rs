use crate::wit::pumpkin::plugin::world::{
    Block, BlockState, get_all_block_names, get_all_blocks, get_block_by_id, get_block_by_name,
    get_block_count, get_block_from_state, get_block_from_state_id, get_block_properties,
    get_block_state_by_id, get_block_state_count, get_default_state_from_block,
    get_default_state_from_block_id, get_state_ids_for_block_id, get_states_for_block,
    get_states_for_block_id,
};

impl Block {
    /// Returns all registered blocks in the registry.
    #[must_use]
    pub fn all() -> Vec<Self> {
        get_all_blocks()
    }

    /// Returns the names of all registered blocks.
    #[must_use]
    pub fn all_names() -> Vec<String> {
        get_all_block_names()
    }

    /// Returns the total number of registered block types.
    #[must_use]
    pub fn count() -> u32 {
        get_block_count()
    }

    /// Returns the total number of registered block states.
    #[must_use]
    pub fn total_state_count() -> u32 {
        get_block_state_count()
    }

    /// Gets a block definition by its numerical block ID.
    #[must_use]
    pub fn from_id(id: u16) -> Option<Self> {
        get_block_by_id(id)
    }

    /// Gets a block definition by its namespaced name (e.g., "minecraft:stone" or "stone").
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        get_block_by_name(name)
    }

    /// Gets the block definition for a given block state ID.
    #[must_use]
    pub fn from_state_id(state_id: u16) -> Option<Self> {
        get_block_from_state_id(state_id)
    }

    /// Gets the block definition for a given block state.
    #[must_use]
    pub fn from_state(state: &BlockState) -> Self {
        get_block_from_state(state)
    }

    /// Gets all valid block states for this block type.
    #[must_use]
    pub fn get_states(&self) -> Vec<BlockState> {
        get_states_for_block(self)
    }

    /// Gets all valid block states for a given numerical block ID.
    #[must_use]
    pub fn get_states_for_id(block_id: u16) -> Vec<BlockState> {
        get_states_for_block_id(block_id)
    }

    /// Gets all valid block state IDs for a given numerical block ID.
    #[must_use]
    pub fn get_state_ids_for_id(block_id: u16) -> Vec<u16> {
        get_state_ids_for_block_id(block_id)
    }

    /// Gets the default block state for this block.
    #[must_use]
    pub fn get_default_state(&self) -> BlockState {
        get_default_state_from_block(self)
    }

    /// Gets the default block state for a given numerical block ID.
    #[must_use]
    pub fn get_default_state_for_id(block_id: u16) -> Option<BlockState> {
        get_default_state_from_block_id(block_id)
    }
}

impl BlockState {
    /// Gets the parent block definition for this block state.
    #[must_use]
    pub fn get_block(&self) -> Block {
        get_block_from_state(self)
    }

    /// Gets a detailed block state by its numerical block state ID.
    #[must_use]
    pub fn from_id(state_id: u16) -> Option<Self> {
        get_block_state_by_id(state_id)
    }

    /// Gets property key-value pairs for this block state.
    #[must_use]
    pub fn get_properties(&self) -> Vec<(String, String)> {
        get_block_properties(self.id)
    }
}
