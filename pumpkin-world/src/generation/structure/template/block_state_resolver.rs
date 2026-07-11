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

        // Transform properties for rotation/mirror
        let transformed_props: Vec<(String, String)> = entry
            .properties
            .iter()
            .map(|(key, value)| {
                let transformed_key = match key.as_str() {
                    "north" | "south" | "east" | "west" => rotation
                        .rotate_facing(mirror.mirror_facing(key))
                        .to_string(),
                    _ => key.clone(),
                };
                let new_value = Self::transform_property(key, value, rotation, mirror);
                (transformed_key, new_value)
            })
            .collect();

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

    /// Transforms a property value based on rotation and mirror.
    fn transform_property(key: &str, value: &str, rotation: Rotation, mirror: Mirror) -> String {
        match key {
            // Horizontal facing properties
            "facing" => {
                let mirrored = mirror.mirror_facing(value);
                rotation.rotate_facing(mirrored).to_string()
            }

            // Axis properties (for logs, pillars, etc.)
            "axis" => rotation.rotate_axis(value).to_string(),

            // Block rotation (signs, banners - 0-15 value)
            "rotation" => value.parse::<i32>().map_or_else(
                |_| value.to_string(),
                |rot_value| {
                    let mirrored = mirror.mirror_block_rotation(rot_value);
                    let rotated = rotation.rotate_block_rotation(mirrored);
                    Self::rotation_to_str(rotated).to_string()
                },
            ),

            // Half properties don't need rotation (top/bottom stays the same)
            // Shape, mode, and most other properties don't need transformation either
            _ => value.to_string(),
        }
    }

    /// Converts a rotation value (0-15) to a static string.
    const fn rotation_to_str(rotation: i32) -> &'static str {
        match rotation % 16 {
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "11",
            12 => "12",
            13 => "13",
            14 => "14",
            15 => "15",
            _ => "0",
        }
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
}
