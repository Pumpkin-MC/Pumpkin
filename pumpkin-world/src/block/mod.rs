pub mod entities;
pub mod state;

use serde::Deserialize;
pub use state::RawBlockState;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStateCodec {
    pub name: String,
    // TODO: properties...
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
