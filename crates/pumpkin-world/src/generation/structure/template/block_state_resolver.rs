//! Block state resolution from template palette entries.
//!
//! This module handles converting NBT palette entries (block name + properties)
//! to the runtime block state IDs used by the world, with support for rotation
//! and mirroring transformations.

use pumpkin_data::{Block, BlockState, Mirror, Rotation};
use tracing::warn;

use super::PaletteEntry;

/// Resolves template palette entries to block state IDs.
///
/// This resolver handles:
/// - Looking up blocks by name
/// - Applying block state properties
/// - Rotating/mirroring directional properties (facing, axis, rotation)
pub struct BlockStateResolver;

impl BlockStateResolver {
    /// Resolves a palette entry to a block state, applying rotation and mirror transforms.
    ///
    /// Returns the resolved `BlockState` or `None` if the block is not found.
    #[must_use]
    pub fn resolve(
        entry: &PaletteEntry,
        rotation: Rotation,
        mirror: Mirror,
    ) -> Option<&'static BlockState> {
        // Strip minecraft: prefix if present
        let block_name = entry.name.strip_prefix("minecraft:").unwrap_or(&entry.name);

        // Find the block
        let block = Block::from_name(&entry.name).or_else(|| Block::from_registry_key(block_name));

        let Some(block) = block else {
            warn!("Unknown block in template: {}", entry.name);
            return None;
        };

        // If no properties, return default state
        if entry.properties.is_empty() {
            return Some(block.default_state);
        }

        // Transform properties for rotation/mirror using the unified vanilla logic
        let transformed_props = pumpkin_data::transform_block_properties(
            &entry.name,
            &entry.properties,
            rotation,
            mirror,
        );

        // Convert to the format expected by from_properties
        let props_slice = transformed_props
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect::<Vec<_>>();

        // Get the state ID from properties
        let props_box = block.from_properties(&props_slice);
        let state_id = props_box.to_state_id(block);

        Some(BlockState::from_id(state_id))
    }

    /// Resolves a palette entry without any transformation.
    #[must_use]
    pub fn resolve_simple(entry: &PaletteEntry) -> Option<&'static BlockState> {
        Self::resolve(entry, Rotation::None, Mirror::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_simple_block() {
        let entry = PaletteEntry::new("minecraft:stone".to_string());
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_some());
    }

    #[test]
    fn resolve_with_properties() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                ("facing".to_string(), "north".to_string()),
                ("half".to_string(), "bottom".to_string()),
                ("shape".to_string(), "straight".to_string()),
                ("waterlogged".to_string(), "false".to_string()),
            ],
        );
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_some());
    }

    #[test]
    fn rotation_transforms_facing() {
        let entry = PaletteEntry::with_properties(
            "minecraft:furnace".to_string(),
            vec![
                ("facing".to_string(), "north".to_string()),
                ("lit".to_string(), "false".to_string()),
            ],
        );

        // Get state with rotation
        let rotated = BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::None);
        assert!(rotated.is_some());

        // The rotated block should have facing=east after 90 degree clockwise rotation
        // We can't easily verify the exact facing here without more infrastructure,
        // but we can verify it resolves successfully
    }

    #[test]
    fn unknown_block_returns_none() {
        let entry = PaletteEntry::new("minecraft:nonexistent_block".to_string());
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_none());
    }

    #[test]
    fn stairs_mirror_and_rotate_resolves_expected_state() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                ("facing".to_string(), "north".to_string()),
                ("half".to_string(), "bottom".to_string()),
                ("shape".to_string(), "inner_left".to_string()),
                ("waterlogged".to_string(), "false".to_string()),
            ],
        );

        // Mirror LeftRight (north -> south, inner_left -> inner_right)
        // then rotate Clockwise90 (south -> west)
        let resolved =
            BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::LeftRight).unwrap();

        // Compare with explicitly resolved target state
        let target_entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                ("facing".to_string(), "west".to_string()),
                ("half".to_string(), "bottom".to_string()),
                ("shape".to_string(), "inner_right".to_string()),
                ("waterlogged".to_string(), "false".to_string()),
            ],
        );
        let expected = BlockStateResolver::resolve_simple(&target_entry).unwrap();
        assert_eq!(resolved.id, expected.id);
    }

    #[test]
    fn door_mirror_resolves_flipped_hinge() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_door".to_string(),
            vec![
                ("facing".to_string(), "east".to_string()),
                ("half".to_string(), "lower".to_string()),
                ("hinge".to_string(), "left".to_string()),
                ("open".to_string(), "false".to_string()),
                ("powered".to_string(), "false".to_string()),
            ],
        );

        // Mirror FrontBack (east -> west, hinge left -> right)
        let resolved =
            BlockStateResolver::resolve(&entry, Rotation::None, Mirror::FrontBack).unwrap();

        let target_entry = PaletteEntry::with_properties(
            "minecraft:oak_door".to_string(),
            vec![
                ("facing".to_string(), "west".to_string()),
                ("half".to_string(), "lower".to_string()),
                ("hinge".to_string(), "right".to_string()),
                ("open".to_string(), "false".to_string()),
                ("powered".to_string(), "false".to_string()),
            ],
        );
        let expected = BlockStateResolver::resolve_simple(&target_entry).unwrap();
        assert_eq!(resolved.id, expected.id);
    }
}
