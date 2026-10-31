pub use crate::keys::BlockType;
use crate::wit::pumpkin::plugin::world::{Block, BlockState};

/// Trait for converting types into a valid Minecraft block registry key (e.g. `"minecraft:stone"`).
pub trait IntoBlockKey {
    /// Returns the namespaced block registry key.
    fn into_block_key(self) -> String;
}

impl IntoBlockKey for BlockType {
    fn into_block_key(self) -> String {
        self.into_registry_key()
    }
}

impl IntoBlockKey for &BlockType {
    fn into_block_key(self) -> String {
        self.resource_location().to_string()
    }
}

impl IntoBlockKey for &str {
    fn into_block_key(self) -> String {
        crate::keys::qualify(self)
    }
}

impl IntoBlockKey for String {
    fn into_block_key(self) -> String {
        crate::keys::qualify(&self)
    }
}

impl IntoBlockKey for &String {
    fn into_block_key(self) -> String {
        self.as_str().into_block_key()
    }
}

/// Extension trait providing typed helper methods on `Block`.
pub trait BlockTypeExt {
    /// Gets a block definition from a typed block, a plain name, or a namespaced key.
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
        Self::from_name(&key.into_block_key())
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
