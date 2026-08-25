pub mod entities;
pub mod viewer;

use std::collections::HashMap;

use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Writes a block state the way vanilla's `BlockState.CODEC` does: a `{Name, Properties}`
/// compound, with `Properties` left out entirely for a block that has none.
///
/// NBT form of the same shape the palette codec uses on disk. Raw state IDs are a
/// build-specific numbering, dense and stable only for one exact block registry. Block
/// entities that carry a state (a piston placeholder's moved block) go through here.
#[must_use]
pub fn block_state_to_nbt(state_id: BlockStateId) -> NbtCompound {
    let block = Block::from_state_id(state_id);
    let mut nbt = NbtCompound::new();
    let name = if block.name.starts_with("minecraft:") {
        block.name.to_string()
    } else {
        format!("minecraft:{}", block.name)
    };
    nbt.put_string("Name", name);

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

/// Reads back what [`block_state_to_nbt`] wrote.
///
/// `None` only when `Name` is missing or names a block this build does not have; unknown or
/// absent properties fall back to the block's own defaults, matching vanilla's codec.
///
/// An unrecognised `Name` is not necessarily corrupt: region files outlive the server, so
/// the name can be a block from an older version that has since been renamed or removed.
/// Vanilla data-fixes this field; Pumpkin has no fixers, so the caller supplies its own
/// default.
#[must_use]
pub fn block_state_from_nbt(nbt: &NbtCompound) -> Option<BlockStateId> {
    let block = Block::from_name(nbt.get_string("Name")?)?;
    let Some(properties) = nbt.get_compound("Properties") else {
        return Some(block.default_state.id);
    };

    let properties: Vec<(&str, &str)> = properties
        .child_tags
        .iter()
        .filter_map(|(key, tag)| Some((&**key, tag.extract_string()?)))
        .collect();
    Some(block.from_properties(&properties).to_state_id(block))
}

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
        let block = self.name;

        let Some(properties_map) = &self.properties else {
            return block.default_state.id;
        };

        let props_iter = properties_map
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect::<Vec<(&str, &str)>>();

        let block_properties = block.from_properties(&props_iter);
        block_properties.to_state_id(block)
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::{Block, BlockStateId, block_properties::BlockProperties};
    use pumpkin_nbt::compound::NbtCompound;

    use super::{block_state_from_nbt, block_state_to_nbt};
    use crate::chunk::palette::BLOCK_NETWORK_MAX_BITS;

    #[test]
    fn block_state_nbt_round_trip() {
        // A property-less block, one with several properties, and the piston head a moving
        // placeholder carries: the case the block-entity codec exists for.
        let mut head =
            pumpkin_data::block_properties::PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
        head.facing = pumpkin_data::block_properties::Facing::East;
        head.short = true;
        head.r#type = pumpkin_data::block_properties::PistonType::Sticky;

        for state_id in [
            Block::STONE.default_state.id,
            Block::SLIME_BLOCK.default_state.id,
            head.to_state_id(&Block::PISTON_HEAD),
        ] {
            let nbt = block_state_to_nbt(state_id);
            assert_eq!(block_state_from_nbt(&nbt), Some(state_id));
        }
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
