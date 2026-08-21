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

/// Resolves a WIT block type handle to a static `Block` reference.
/// Returns `Err` if the block ID is out of range, preventing a host crash.
fn bt_block(bt: WitBlockType) -> wasmtime::Result<&'static pumpkin_data::Block> {
    pumpkin_data::BlockId::new(bt.id)
        .ok_or_else(|| wasmtime::Error::msg("invalid block id"))
        .map(pumpkin_data::BlockId::to_block)
}

impl Host for PluginHostState {
    /// Looks up a block type by its registry name.
    /// `Block::from_name` strips the `"minecraft:"` prefix internally,
    /// so both `"minecraft:oak_slab"` and `"oak_slab"` are accepted.
    async fn block_type_from_name(
        &mut self,
        name: String,
    ) -> wasmtime::Result<Option<WitBlockType>> {
        if name.is_empty() || name.len() > MAX_STRING_LEN {
            return Ok(None);
        }
        Ok(pumpkin_data::Block::from_name(&name).map(|b| WitBlockType { id: b.id.as_u16() }))
    }

    /// Returns the full namespaced registry name (e.g., `"minecraft:oak_slab"`).
    /// Uses the existing `ToResourceLocation` trait on `Block`.
    async fn block_type_name(&mut self, bt: WitBlockType) -> wasmtime::Result<String> {
        Ok(bt_block(bt)?.to_resource_location())
    }

    /// Returns the registry name with underscores replaced by spaces.
    /// This is not a localized display name; use the text interface for that.
    async fn block_type_display_name(&mut self, bt: WitBlockType) -> wasmtime::Result<String> {
        Ok(bt_block(bt)?.name.replace('_', " "))
    }

    /// Whether this block type is solid by default.
    /// Delegates to `Block::is_solid()` which checks the default state's bitflags.
    async fn block_type_is_solid(&mut self, bt: WitBlockType) -> wasmtime::Result<bool> {
        Ok(bt_block(bt)?.is_solid())
    }

    /// Whether this block type is air.
    /// Delegates to `Block::is_air()` which checks the default state's bitflags.
    async fn block_type_is_air(&mut self, bt: WitBlockType) -> wasmtime::Result<bool> {
        Ok(bt_block(bt)?.is_air())
    }

    /// Whether this block type is a liquid (water or lava, including flowing variants).
    /// Uses `BlockState::is_liquid()` on the default state.
    async fn block_type_is_liquid(&mut self, bt: WitBlockType) -> wasmtime::Result<bool> {
        Ok(bt_block(bt)?.default_state.is_liquid())
    }

    /// Whether this block type can catch fire.
    /// `Block.flammable` is `Some` if the block has fire behavior defined.
    async fn block_type_is_flammable(&mut self, bt: WitBlockType) -> wasmtime::Result<bool> {
        Ok(bt_block(bt)?.flammable.is_some())
    }

    /// Mining hardness from the default state.
    /// Returns `0.0` if the value is NaN or infinity to prevent plugin-side corruption.
    async fn block_type_hardness(&mut self, bt: WitBlockType) -> wasmtime::Result<f32> {
        let h = bt_block(bt)?.hardness;
        Ok(if h.is_finite() { h } else { 0.0 })
    }

    /// Resistance to explosions from the default state.
    /// Returns `0.0` if the value is NaN or infinity to prevent plugin-side corruption.
    async fn block_type_blast_resistance(&mut self, bt: WitBlockType) -> wasmtime::Result<f32> {
        let r = bt_block(bt)?.blast_resistance;
        Ok(if r.is_finite() { r } else { 0.0 })
    }

    /// Light emission level (0-15) from the default state.
    /// For blocks with state-dependent luminance (e.g., redstone wire),
    /// this returns the default state's value only.
    async fn block_type_light_emission(&mut self, bt: WitBlockType) -> wasmtime::Result<u8> {
        Ok(bt_block(bt)?.default_state.luminance)
    }

    /// Checks if a block type belongs to a given tag.
    /// Strips the `"minecraft:"` prefix from the tag name, then looks up
    /// the tag's member ID list and checks for containment.
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
    /// Useful for validating tag names before passing them to `block-type-has-tag`.
    async fn is_valid_block_tag(&mut self, tag: String) -> wasmtime::Result<bool> {
        if tag.is_empty() || tag.len() > MAX_STRING_LEN {
            return Ok(false);
        }
        let bare = tag.strip_prefix("minecraft:").unwrap_or(&tag);
        Ok(tag::get_tag_ids(RegistryKey::Block, bare).is_some())
    }

    /// Returns all block types that belong to a given tag.
    /// Results are capped at `MAX_TAG_MEMBERS` to prevent memory exhaustion.
    /// Returns an empty list if the tag doesn't exist.
    async fn get_block_tag_members(&mut self, tag: String) -> wasmtime::Result<Vec<WitBlockType>> {
        if tag.is_empty() || tag.len() > MAX_STRING_LEN {
            return Ok(Vec::new());
        }
        let bare = tag.strip_prefix("minecraft:").unwrap_or(&tag);
        Ok(tag::get_tag_ids(RegistryKey::Block, bare)
            .map(|ids| {
                ids.iter()
                    .take(MAX_TAG_MEMBERS)
                    .map(|&id| WitBlockType { id })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Returns the default state ID for a block type.
    /// This is the state placed when a player puts the block without specifying properties.
    async fn block_type_default_state_id(&mut self, bt: WitBlockType) -> wasmtime::Result<u16> {
        Ok(bt_block(bt)?.default_state.id.as_u16())
    }

    /// Returns all possible state IDs for a block type.
    /// Capped at `MAX_STATE_IDS` to prevent memory exhaustion.
    async fn block_type_all_state_ids(&mut self, bt: WitBlockType) -> wasmtime::Result<Vec<u16>> {
        Ok(bt_block(bt)?
            .states
            .iter()
            .take(MAX_STATE_IDS)
            .map(|s| s.id.as_u16())
            .collect())
    }

    /// Converts a block state ID to its parent block type.
    /// Uses a pre-computed array for O(1) lookup.
    async fn state_id_to_block_type(&mut self, state_id: u16) -> wasmtime::Result<WitBlockType> {
        let state_id = pumpkin_data::BlockStateId::new(state_id)
            .ok_or_else(|| wasmtime::Error::msg("invalid state id"))?;
        Ok(WitBlockType {
            id: pumpkin_data::BlockId::from_state_id(state_id).as_u16(),
        })
    }
}
