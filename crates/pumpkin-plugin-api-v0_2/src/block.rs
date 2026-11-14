use crate::wit::pumpkin::plugin::world::{Block, BlockState};

/// Trait for converting types into a valid Minecraft block registry key (e.g. `"minecraft:stone"`).
pub trait IntoBlockKey {
    /// Returns the namespaced block registry key.
    fn into_block_key(self) -> String;
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

/// Extension trait providing helper methods on `Block`.
pub trait BlockTypeExt {
    /// Gets a block definition from a plain name or a namespaced registry key.
    #[must_use]
    fn of(key: impl IntoBlockKey) -> Option<Block>;

    /// Checks if this block matches the given block key.
    #[must_use]
    fn matches_block(&self, key: impl IntoBlockKey) -> bool;
}

impl BlockTypeExt for Block {
    fn of(key: impl IntoBlockKey) -> Option<Block> {
        Self::from_name(&key.into_block_key())
    }

    fn matches_block(&self, key: impl IntoBlockKey) -> bool {
        let expected = key.into_block_key();
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = self.name.strip_prefix("minecraft:").unwrap_or(&self.name);
        expected_clean == actual_clean
    }
}

/// Extension trait providing helper methods on `BlockState`.
pub trait BlockStateTypeExt {
    /// Checks if this block state matches the given block key.
    #[must_use]
    fn matches_block(&self, key: impl IntoBlockKey) -> bool;
}

impl BlockStateTypeExt for BlockState {
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
