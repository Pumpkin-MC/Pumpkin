use pumpkin_data::tag::{self, RegistryKey};
use pumpkin_util::resource_location::ToResourceLocation;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::plugin::blocks::{BlockType as WitBlockType, Host},
};

/// Maximum number of block types returned by `get-block-tag-members`.
/// Prevents unbounded memory allocation when enumerating large tags.
const MAX_TAG_MEMBERS: usize = 256;

/// Maximum number of state IDs returned by `block-type-all-state-ids`.
/// Prevents unbounded memory allocation for blocks with many states.
const MAX_STATE_IDS: usize = 256;

/// Maximum length for string inputs (block names, tag names).
/// The longest legitimate Minecraft name is 45 characters.
const MAX_STRING_LEN: usize = 64;

/// Builds a full [`WitBlockType`] record from a [`pumpkin_data::Block`] reference.
/// All fields are populated from the block's static metadata and its default state.
fn block_to_wit(block: &'static pumpkin_data::Block) -> WitBlockType {
    let ds = block.default_state;
    WitBlockType {
        id: block.id.as_u16(),
        name: block.to_resource_location(),
        display_name: block.name.replace('_', " "),
        hardness: if block.hardness.is_finite() {
            block.hardness
        } else {
            0.0
        },
        blast_resistance: if block.blast_resistance.is_finite() {
            block.blast_resistance
        } else {
            0.0
        },
        map_color: block.map_color,
        is_solid: block.is_solid(),
        is_air: block.is_air(),
        is_liquid: ds.is_liquid(),
        burnable: block.flammable.is_some(),
        light_emission: ds.luminance,
        default_state_id: ds.id.as_u16(),
    }
}

impl Host for PluginHostState {
    /// Looks up a block type by its registry name and returns the full record.
    /// `Block::from_name` strips the `"minecraft:"` prefix internally,
    /// so both `"minecraft:oak_slab"` and `"oak_slab"` are accepted.
    async fn get_block_type(&mut self, name: String) -> wasmtime::Result<Option<WitBlockType>> {
        if name.is_empty() || name.len() > MAX_STRING_LEN {
            return Ok(None);
        }
        Ok(pumpkin_data::Block::from_name(&name).map(block_to_wit))
    }

    /// Converts a block state ID to its parent block type, returning the full record.
    async fn block_type_from_state_id(
        &mut self,
        state_id: u16,
    ) -> wasmtime::Result<Option<WitBlockType>> {
        let Some(state_id) = pumpkin_data::BlockStateId::new(state_id) else {
            return Ok(None);
        };
        let block = pumpkin_data::Block::from_state_id(state_id);
        Ok(Some(block_to_wit(block)))
    }

    /// Checks if a block type belongs to a given tag.
    async fn block_type_has_tag(
        &mut self,
        bt: WitBlockType,
        tag: String,
    ) -> wasmtime::Result<bool> {
        if tag.is_empty() || tag.len() > MAX_STRING_LEN {
            return Ok(false);
        }
        let bare = tag.strip_prefix("minecraft:").unwrap_or(&tag);
        Ok(tag::get_tag_ids(RegistryKey::Block, bare).is_some_and(|ids| ids.contains(&bt.id)))
    }

    /// Checks if a tag with the given name exists in the block registry.
    async fn is_valid_block_tag(&mut self, tag: String) -> wasmtime::Result<bool> {
        if tag.is_empty() || tag.len() > MAX_STRING_LEN {
            return Ok(false);
        }
        let bare = tag.strip_prefix("minecraft:").unwrap_or(&tag);
        Ok(tag::get_tag_ids(RegistryKey::Block, bare).is_some())
    }

    /// Returns all block types that belong to a given tag.
    /// Results are capped at `MAX_TAG_MEMBERS` to prevent memory exhaustion.
    async fn get_block_tag_members(&mut self, tag: String) -> wasmtime::Result<Vec<WitBlockType>> {
        if tag.is_empty() || tag.len() > MAX_STRING_LEN {
            return Ok(Vec::new());
        }
        let bare = tag.strip_prefix("minecraft:").unwrap_or(&tag);
        Ok(tag::get_tag_ids(RegistryKey::Block, bare)
            .map(|ids| {
                ids.iter()
                    .take(MAX_TAG_MEMBERS)
                    .filter_map(|&id| {
                        pumpkin_data::BlockId::new(id).map(|bid| block_to_wit(bid.to_block()))
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Returns all possible state IDs for a block type.
    /// Capped at `MAX_STATE_IDS` to prevent memory exhaustion.
    async fn block_type_all_state_ids(&mut self, bt: WitBlockType) -> wasmtime::Result<Vec<u16>> {
        let block = pumpkin_data::BlockId::new(bt.id)
            .ok_or_else(|| wasmtime::Error::msg("invalid block id"))?
            .to_block();
        Ok(block
            .states
            .iter()
            .take(MAX_STATE_IDS)
            .map(|s| s.id.as_u16())
            .collect())
    }
}
