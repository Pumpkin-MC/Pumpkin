pub use crate::generated::item::Item;
use crate::wit::pumpkin::plugin::item_stack::ItemStack;

/// Trait for converting types into a valid Minecraft item registry key (e.g. `"minecraft:diamond"`).
pub trait IntoItemKey {
    /// Returns the namespaced item registry key.
    fn into_item_key(self) -> String;
}

impl IntoItemKey for Item {
    fn into_item_key(self) -> String {
        self.resource_location().to_string()
    }
}

impl IntoItemKey for &Item {
    fn into_item_key(self) -> String {
        self.resource_location().to_string()
    }
}

impl IntoItemKey for &str {
    fn into_item_key(self) -> String {
        if self.contains(':') {
            self.to_string()
        } else {
            format!("minecraft:{self}")
        }
    }
}

impl IntoItemKey for String {
    fn into_item_key(self) -> String {
        if self.contains(':') {
            self
        } else {
            format!("minecraft:{self}")
        }
    }
}

impl IntoItemKey for &String {
    fn into_item_key(self) -> String {
        self.as_str().into_item_key()
    }
}

/// Extension trait providing typed helper constructors and utilities on `ItemStack`.
pub trait ItemStackExt {
    /// Creates a new `ItemStack` from any valid item key (e.g. `Item::Diamond`, `"minecraft:diamond"`, or `"custom:item"`).
    #[must_use]
    fn of(item: impl IntoItemKey, count: u8) -> Self;

    /// Returns the typed `Item` enum if this item stack corresponds to a known vanilla item,
    /// or `None` if it is a custom, modded, or newer unknown item.
    #[must_use]
    fn get_item(&self) -> Option<Item>;

    /// Returns whether this item matches the specified typed `Item` enum.
    #[must_use]
    fn is_item(&self, item: Item) -> bool;

    /// Checks if this item stack matches a given item (either `Item` enum or string identifier).
    #[must_use]
    fn matches_item(&self, item: impl IntoItemKey) -> bool;
}

impl ItemStackExt for ItemStack {
    fn of(item: impl IntoItemKey, count: u8) -> Self {
        Self::new(&item.into_item_key(), count)
    }

    fn get_item(&self) -> Option<Item> {
        Item::from_registry_key(&self.get_registry_key())
    }

    fn is_item(&self, item: Item) -> bool {
        self.get_item() == Some(item)
    }

    fn matches_item(&self, item: impl IntoItemKey) -> bool {
        let expected = item.into_item_key();
        let actual = self.get_registry_key();
        let expected_clean = expected.strip_prefix("minecraft:").unwrap_or(&expected);
        let actual_clean = actual.strip_prefix("minecraft:").unwrap_or(&actual);
        expected_clean == actual_clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_to_resource_location() {
        assert_eq!(Item::Diamond.into_item_key(), "minecraft:diamond");
        assert_eq!(
            Item::NetheriteSword.into_item_key(),
            "minecraft:netherite_sword"
        );
        assert_eq!(Item::AcaciaBoat.into_item_key(), "minecraft:acacia_boat");
        assert_eq!(
            Item::AllaySpawnEgg.into_item_key(),
            "minecraft:allay_spawn_egg"
        );
        assert_eq!(Item::Tnt.into_item_key(), "minecraft:tnt");
    }

    #[test]
    fn string_to_resource_location() {
        assert_eq!("diamond".into_item_key(), "minecraft:diamond");
        assert_eq!("minecraft:diamond".into_item_key(), "minecraft:diamond");
        assert_eq!("custom:laser_gun".into_item_key(), "custom:laser_gun");
    }

    #[test]
    fn item_parsing() {
        assert_eq!(Item::from_name("diamond"), Some(Item::Diamond));
        assert_eq!(Item::from_name("minecraft:diamond"), Some(Item::Diamond));
        assert_eq!(
            Item::from_name("netherite_sword"),
            Some(Item::NetheriteSword)
        );
        assert_eq!(Item::from_name("custom:magic_wand"), None);
    }
}
