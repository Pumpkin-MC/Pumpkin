#![allow(async_fn_in_trait)]

pub use crate::keys::Item;
use crate::wit::pumpkin::plugin::item_stack::ItemStack;

/// Trait for converting types into a valid Minecraft item registry key (e.g. `"minecraft:diamond"`).
pub trait IntoItemKey {
    /// Returns the namespaced item registry key.
    fn into_item_key(self) -> String;
}

impl IntoItemKey for Item {
    fn into_item_key(self) -> String {
        self.into_registry_key()
    }
}

impl IntoItemKey for &Item {
    fn into_item_key(self) -> String {
        self.resource_location().to_string()
    }
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

/// Extension trait providing typed helper constructors and utilities on `ItemStack`.
pub trait ItemStackExt {
    /// Creates an item stack from a typed item, a plain name, or a namespaced key.
    #[must_use]
    fn of(item: impl IntoItemKey, count: u8) -> Self;

    /// Returns the typed `Item` key if this item stack corresponds to a known vanilla item,
    /// or `None` if it is a custom, modded, or newer unknown item.
    #[must_use]
    async fn get_item(&self) -> Option<Item>;

    /// Returns whether this item matches the specified typed `Item` key.
    #[must_use]
    async fn is_item(&self, item: Item) -> bool;

    /// Checks if this item stack matches a given item (either an `Item` key or string identifier).
    #[must_use]
    async fn matches_item(&self, item: impl IntoItemKey) -> bool;
}

impl ItemStackExt for ItemStack {
    fn of(item: impl IntoItemKey, count: u8) -> Self {
        Self::new(&item.into_item_key(), count)
    }

    async fn get_item(&self) -> Option<Item> {
        Item::from_registry_key(&self.get_registry_key().await)
    }

    async fn is_item(&self, item: Item) -> bool {
        self.get_item().await == Some(item)
    }

    async fn matches_item(&self, item: impl IntoItemKey) -> bool {
        let expected = item.into_item_key();
        let actual = self.get_registry_key().await;
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = actual.strip_prefix("minecraft:").unwrap_or(&actual);
        expected_clean == actual_clean
    }
}
