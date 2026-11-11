#![allow(async_fn_in_trait)]

use crate::wit::pumpkin::plugin::item_stack::ItemStack;

/// Trait for converting types into a valid Minecraft item registry key (e.g. `"minecraft:diamond"`).
pub trait IntoItemKey {
    /// Returns the namespaced item registry key.
    fn into_item_key(self) -> String;
}

impl IntoItemKey for &str {
    fn into_item_key(self) -> String {
        crate::keys::qualify(self)
    }
}

impl IntoItemKey for String {
    fn into_item_key(self) -> String {
        crate::keys::qualify(&self)
    }
}

impl IntoItemKey for &String {
    fn into_item_key(self) -> String {
        self.as_str().into_item_key()
    }
}

/// Extension trait providing helper constructors and utilities on `ItemStack`.
pub trait ItemStackExt {
    /// Creates an item stack from a plain name or a namespaced registry key.
    #[must_use]
    fn of(item: impl IntoItemKey, count: u8) -> Self;

    /// Checks if this item stack matches a given item (a plain name or namespaced identifier).
    #[must_use]
    async fn matches_item(&self, item: impl IntoItemKey) -> bool;
}

impl ItemStackExt for ItemStack {
    fn of(item: impl IntoItemKey, count: u8) -> Self {
        Self::new(&item.into_item_key(), count)
    }

    async fn matches_item(&self, item: impl IntoItemKey) -> bool {
        let expected = item.into_item_key();
        let actual = self.get_registry_key().await;
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = actual.strip_prefix("minecraft:").unwrap_or(&actual);
        expected_clean == actual_clean
    }
}
