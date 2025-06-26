pub mod entities;
pub mod state;

use std::collections::HashMap;

use pumpkin_data::{
    BlockState,
    block_properties::{get_block, get_state_by_state_id},
};
use serde::{Deserialize, Serialize};
pub use state::RawBlockState;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStateCodec {
    /// Block name
    pub name: String,
    /// Key-value pairs of properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
}

impl BlockStateCodec {
    pub fn get_state(&self) -> Option<BlockState> {
        let block = get_block(self.name.as_str())?;

        let mut state_id = block.default_state.id;

        if let Some(properties) = &self.properties {
            let props = properties
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let block_properties = block.from_properties(props).unwrap();
            state_id = block_properties.to_state_id(&block);
        }

        get_state_by_state_id(state_id)
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::Block;

    use crate::chunk::palette::BLOCK_NETWORK_MAX_BITS;

    #[test]
    fn test_proper_network_bits_per_entry() {
        let id_to_test = 1 << BLOCK_NETWORK_MAX_BITS;
        if Block::from_state_id(id_to_test).is_some() {
            panic!("We need to update our constants!");
        }
    }
}
