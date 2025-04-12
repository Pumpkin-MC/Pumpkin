use crate::chunk::format::PaletteBlockEntry;

use super::registry::{get_block, get_block_by_state_id, get_state_by_state_id};

/// Instead of using a memory heavy normal BlockState This is used for internal representation in chunks to save memory
#[derive(Clone, Copy, Debug, Eq)]
pub struct RawBlockState {
    pub state_id: u16,
}

impl PartialEq for RawBlockState {
    fn eq(&self, other: &Self) -> bool {
        self.state_id == other.state_id
    }
}

impl RawBlockState {
    pub const AIR: RawBlockState = RawBlockState { state_id: 0 };

    /// Get a Block from the Vanilla Block registry at Runtime
    pub fn new(registry_id: &str) -> Option<Self> {
        let block = get_block(registry_id);
        block.map(|block| Self {
            state_id: block.default_state_id,
        })
    }

    pub fn from_palette(palette: &PaletteBlockEntry) -> Option<Self> {
        let block = get_block(palette.name.as_str());

        if let Some(block) = block {
            let mut state_id = block.default_state_id;

            if let Some(properties) = palette.properties.clone() {
                let mut properties_vec = Vec::new();
                for (key, value) in properties {
                    properties_vec.push((key.clone(), value.clone()));
                }
                let block_properties = block.from_properties(properties_vec).unwrap();
                state_id = block_properties.to_state_id(&block);
            }

            return Some(Self { state_id });
        }

        None
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    pub fn to_state(&self) -> pumpkin_data::block::BlockState {
        get_state_by_state_id(self.state_id).unwrap()
    }

    pub fn to_block(&self) -> pumpkin_data::block::Block {
        get_block_by_state_id(self.state_id).unwrap()
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
