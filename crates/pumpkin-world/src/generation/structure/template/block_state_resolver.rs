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
        let state_id = if rotation == Rotation::None && mirror == Mirror::None {
            block.from_properties(&entry.properties).to_state_id(block)
        } else {
            let transformed_props = pumpkin_data::transform_block_properties(
                &entry.name,
                &entry.properties,
                rotation,
                mirror,
            );
            block.from_properties(&transformed_props).to_state_id(block)
        };

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
    use pumpkin_data::block_properties::{
        DoorHinge, DoubleBlockHalf, Facing, Half, Property, StairsShape,
    };

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
                Property::Facing(Facing::North),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::Straight),
                Property::Waterlogged(false),
            ],
        );
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_some());
    }

    #[test]
    fn rotation_transforms_facing() {
        let entry = PaletteEntry::with_properties(
            "minecraft:furnace".to_string(),
            vec![Property::Facing(Facing::North), Property::Lit(false)],
        );

        // Get state with rotation
        let rotated = BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::None);
        assert!(rotated.is_some());
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
                Property::Facing(Facing::North),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::InnerLeft),
                Property::Waterlogged(false),
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
                Property::Facing(Facing::West),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::InnerRight),
                Property::Waterlogged(false),
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
                Property::Facing(Facing::East),
                Property::DoubleBlockHalf(DoubleBlockHalf::Lower),
                Property::Hinge(DoorHinge::Left),
                Property::Open(false),
                Property::Powered(false),
            ],
        );

        // Mirror FrontBack (east -> west, hinge left -> right)
        let resolved =
            BlockStateResolver::resolve(&entry, Rotation::None, Mirror::FrontBack).unwrap();

        let target_entry = PaletteEntry::with_properties(
            "minecraft:oak_door".to_string(),
            vec![
                Property::Facing(Facing::West),
                Property::DoubleBlockHalf(DoubleBlockHalf::Lower),
                Property::Hinge(DoorHinge::Right),
                Property::Open(false),
                Property::Powered(false),
            ],
        );
        let expected = BlockStateResolver::resolve_simple(&target_entry).unwrap();
        assert_eq!(resolved.id, expected.id);
    }
}
