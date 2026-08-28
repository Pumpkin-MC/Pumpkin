pub mod entities;
pub mod viewer;

use std::collections::HashMap;

use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::resource_location::ToResourceLocation;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Vanilla `BlockState.CODEC` NBT `{Name, Properties}`. See [`BlockStateCodec::to_nbt`].
#[must_use]
pub fn block_state_to_nbt(state_id: BlockStateId) -> NbtCompound {
    BlockStateCodec::to_nbt(state_id)
}

/// Inverse of [`block_state_to_nbt`]. See [`BlockStateCodec::from_nbt`].
#[must_use]
pub fn block_state_from_nbt(nbt: &NbtCompound) -> Option<BlockStateId> {
    BlockStateCodec::from_nbt(nbt)
}

/// Vanilla `BlockState.CODEC`: `{Name, Properties}`.
///
/// JSON via serde (worldgen). NBT via [`Self::to_nbt`] / [`Self::from_nbt`] (chunk palettes,
/// piston and falling-block entities). State IDs are build-specific; names survive a registry
/// reshuffle. Pumpkin has no data-fixers, so an unknown `Name` is the caller's default.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStateCodec {
    /// Block name
    #[serde(
        deserialize_with = "parse_block_name",
        serialize_with = "block_to_string"
    )]
    pub name: &'static Block,
    /// Key-value pairs of properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
}

fn parse_block_name<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<&'static Block, D::Error> {
    let s = String::deserialize(deserializer)?;
    let block =
        Block::from_name(s.as_str()).ok_or(serde::de::Error::custom("Invalid block name"))?;
    Ok(block)
}

fn block_to_string<S: Serializer>(block: &'static Block, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(block.name)
}

impl BlockStateCodec {
    /// `{Name, Properties}` NBT. Omits `Properties` when the block has none.
    #[must_use]
    pub fn to_nbt(state_id: BlockStateId) -> NbtCompound {
        let block = Block::from_state_id(state_id);
        let mut nbt = NbtCompound::new();
        nbt.put_string("Name", block.to_resource_location());

        if let Some(properties) = block.properties(state_id) {
            let properties = properties.to_props();
            if !properties.is_empty() {
                let mut compound = NbtCompound::new();
                for (key, value) in properties {
                    compound.put_string(key, value.to_string());
                }
                nbt.put_compound("Properties", compound);
            }
        }
        nbt
    }

    /// Inverse of [`Self::to_nbt`]. `None` if `Name` is missing or unknown this build.
    /// Unknown or absent properties fall back to the block's defaults (vanilla codec).
    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Option<BlockStateId> {
        let block = Block::from_name(nbt.get_string("Name")?)?;
        let Some(properties) = nbt.get_compound("Properties") else {
            return Some(block.default_state.id);
        };

        let properties: Vec<(&str, &str)> = properties
            .child_tags
            .iter()
            .filter_map(|(key, tag)| Some((&**key, tag.extract_string()?)))
            .collect();
        Some(Self::state_id_from_props(block, &properties))
    }

    fn state_id_from_props(block: &'static Block, properties: &[(&str, &str)]) -> BlockStateId {
        block.from_properties(properties).to_state_id(block)
    }

    #[must_use]
    pub fn get_state(&self) -> &'static BlockState {
        let state_id = self.get_state_id();
        BlockState::from_id(state_id)
    }

    #[must_use]
    pub const fn get_block(&self) -> &'static Block {
        self.name
    }

    /// Prefer this over `get_state` when the only the state ID is needed
    #[must_use]
    pub fn get_state_id(&self) -> BlockStateId {
        let Some(properties_map) = &self.properties else {
            return self.name.default_state.id;
        };

        let properties: Vec<(&str, &str)> = properties_map
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        Self::state_id_from_props(self.name, &properties)
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::{Block, BlockStateId};
    use pumpkin_nbt::compound::NbtCompound;

    use super::{block_state_from_nbt, block_state_to_nbt};
    use crate::chunk::palette::BLOCK_NETWORK_MAX_BITS;

    #[test]
    fn block_state_nbt_omits_empty_properties() {
        let nbt = block_state_to_nbt(Block::STONE.default_state.id);
        assert_eq!(nbt.get_string("Name"), Some("minecraft:stone"));
        assert!(nbt.get_compound("Properties").is_none());
    }

    #[test]
    fn block_state_nbt_rejects_unknown_name() {
        let mut nbt = NbtCompound::new();
        nbt.put_string("Name", "minecraft:not_a_real_block".to_string());
        assert!(block_state_from_nbt(&nbt).is_none());
        assert!(block_state_from_nbt(&NbtCompound::new()).is_none());
    }

    #[test]
    fn proper_network_bits_per_entry() {
        let id_to_test = 1 << BLOCK_NETWORK_MAX_BITS;
        assert!(
            BlockStateId::new_or_air(id_to_test) == BlockStateId::AIR,
            "We need to update our constants!"
        );
    }
}
