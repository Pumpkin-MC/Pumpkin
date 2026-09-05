pub use crate::generated::block::BlockType;
use crate::wit::pumpkin::plugin::world::{Block, BlockState};

/// Trait for converting types into a valid Minecraft block registry key (e.g. `"minecraft:stone"`).
pub trait IntoBlockKey {
    /// Returns the namespaced block registry key.
    fn into_block_key(self) -> String;
}

impl IntoBlockKey for BlockType {
    fn into_block_key(self) -> String {
        self.resource_location().to_string()
    }
}

impl IntoBlockKey for &BlockType {
    fn into_block_key(self) -> String {
        self.resource_location().to_string()
    }
}

impl IntoBlockKey for &str {
    fn into_block_key(self) -> String {
        if self.contains(':') {
            self.to_string()
        } else {
            format!("minecraft:{self}")
        }
    }
}

impl IntoBlockKey for String {
    fn into_block_key(self) -> String {
        if self.contains(':') {
            self
        } else {
            format!("minecraft:{self}")
        }
    }
}

impl IntoBlockKey for &String {
    fn into_block_key(self) -> String {
        self.as_str().into_block_key()
    }
}

/// Extension trait providing typed helper methods on `Block`.
pub trait BlockTypeExt {
    /// Gets a block definition by any valid block key (e.g. `BlockType::Stone`, `"minecraft:stone"`, or `"custom:block"`).
    #[must_use]
    fn of(key: impl IntoBlockKey) -> Option<Block>;

    /// Returns the typed `BlockType` if this block corresponds to a known vanilla block,
    /// or `None` if it is a custom, modded, or newer unknown block.
    #[must_use]
    fn get_block_type(&self) -> Option<BlockType>;

    /// Checks if this block matches the given `BlockType`.
    #[must_use]
    fn is_block_type(&self, block_type: BlockType) -> bool;

    /// Checks if this block matches the given block key.
    #[must_use]
    fn matches_block(&self, key: impl IntoBlockKey) -> bool;
}

impl BlockTypeExt for Block {
    fn of(key: impl IntoBlockKey) -> Option<Block> {
        Block::from_name(&key.into_block_key())
    }

    fn get_block_type(&self) -> Option<BlockType> {
        BlockType::from_registry_key(&self.name)
    }

    fn is_block_type(&self, block_type: BlockType) -> bool {
        self.get_block_type() == Some(block_type)
    }

    fn matches_block(&self, key: impl IntoBlockKey) -> bool {
        let expected = key.into_block_key();
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = self.name.strip_prefix("minecraft:").unwrap_or(&self.name);
        expected_clean == actual_clean
    }
}

/// Extension trait providing typed helper methods on `BlockState`.
pub trait BlockStateTypeExt {
    /// Returns the typed `BlockType` if this block state corresponds to a known vanilla block,
    /// or `None` if it is a custom, modded, or newer unknown block.
    #[must_use]
    fn get_block_type(&self) -> Option<BlockType>;

    /// Checks if this block state matches the given `BlockType`.
    #[must_use]
    fn is_block_type(&self, block_type: BlockType) -> bool;

    /// Checks if this block state matches the given block key.
    #[must_use]
    fn matches_block(&self, key: impl IntoBlockKey) -> bool;
}

impl BlockStateTypeExt for BlockState {
    fn get_block_type(&self) -> Option<BlockType> {
        BlockType::from_registry_key(&self.block_name)
    }

    fn is_block_type(&self, block_type: BlockType) -> bool {
        self.get_block_type() == Some(block_type)
    }

    fn matches_block(&self, key: impl IntoBlockKey) -> bool {
        let expected = key.into_block_key();
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = self
            .block_name
            .strip_prefix("minecraft:")
            .unwrap_or(&self.block_name);
        expected_clean == actual_clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_to_resource_location() {
        assert_eq!(BlockType::Stone.into_block_key(), "minecraft:stone");
        assert_eq!(
            BlockType::DiamondBlock.into_block_key(),
            "minecraft:diamond_block"
        );
        assert_eq!(BlockType::OakLog.into_block_key(), "minecraft:oak_log");
        assert_eq!(BlockType::Obsidian.into_block_key(), "minecraft:obsidian");
    }

    #[test]
    fn string_to_resource_location() {
        assert_eq!("stone".into_block_key(), "minecraft:stone");
        assert_eq!("minecraft:stone".into_block_key(), "minecraft:stone");
        assert_eq!("custom:ruby_block".into_block_key(), "custom:ruby_block");
    }

    #[test]
    fn block_parsing() {
        assert_eq!(BlockType::from_name("stone"), Some(BlockType::Stone));
        assert_eq!(
            BlockType::from_name("minecraft:stone"),
            Some(BlockType::Stone)
        );
        assert_eq!(
            BlockType::from_name("diamond_block"),
            Some(BlockType::DiamondBlock)
        );
        assert_eq!(BlockType::from_name("custom:ruby_block"), None);
    }
}
