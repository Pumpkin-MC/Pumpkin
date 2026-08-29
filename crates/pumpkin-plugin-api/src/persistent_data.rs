//! Persistent custom data container API (Bukkit-style `PersistentDataHolder`).
//!
//! Provides namespaced persistent data storage on entities, block entities, chunks,
//! worlds, and item stacks that is automatically persisted to disk in NBT format.

use crate::wit::pumpkin::plugin::block_entity::BlockEntity;
use crate::wit::pumpkin::plugin::common::{NbtTag, NbtTree};
use crate::wit::pumpkin::plugin::item_stack::ItemStack;
use crate::wit::pumpkin::plugin::player::Player;
use crate::wit::pumpkin::plugin::world::{Chunk, Entity, World};

/// Constructs an `NbtTree` containing a single string value.
pub fn string_tree(val: &str) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::StringTag(val.to_string())],
    }
}

/// Constructs an `NbtTree` containing a single 32-bit integer value.
pub fn int_tree(val: i32) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Int(val)],
    }
}

/// Constructs an `NbtTree` containing a single 64-bit integer value.
pub fn long_tree(val: i64) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Long(val)],
    }
}

/// Constructs an `NbtTree` containing a single 16-bit short value.
pub fn short_tree(val: i16) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Short(val)],
    }
}

/// Constructs an `NbtTree` containing a single byte (8-bit) value.
pub fn byte_tree(val: i8) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Byte(val)],
    }
}

/// Constructs an `NbtTree` containing a single boolean value.
pub fn bool_tree(val: bool) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Byte(if val { 1 } else { 0 })],
    }
}

/// Constructs an `NbtTree` containing a single 32-bit float value.
pub fn float_tree(val: f32) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Float(val)],
    }
}

/// Constructs an `NbtTree` containing a single 64-bit double value.
pub fn double_tree(val: f64) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::Double(val)],
    }
}

/// Constructs an `NbtTree` containing a byte array value.
pub fn byte_array_tree(val: Vec<i8>) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::ByteArray(val)],
    }
}

/// Constructs an `NbtTree` containing an integer array value.
pub fn int_array_tree(val: Vec<i32>) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::IntArray(val)],
    }
}

/// Constructs an `NbtTree` containing a long array value.
pub fn long_array_tree(val: Vec<i64>) -> NbtTree {
    NbtTree {
        root: 0,
        tags: vec![NbtTag::LongArray(val)],
    }
}

/// Trait for objects that can hold persistent, namespaced custom NBT data.
///
/// Analogous to Bukkit's `PersistentDataHolder` interface.
pub trait PersistentDataHolder {
    /// Sets a raw NBT tree under the specified namespace and key.
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree);

    /// Gets a raw NBT tree under the specified namespace and key, if present.
    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree>;

    /// Removes custom data stored under the specified namespace and key.
    fn remove_custom_data(&self, namespace: &str, key: &str);

    /// Returns `true` if custom data is stored under the specified namespace and key.
    fn has_custom_data(&self, namespace: &str, key: &str) -> bool;

    /// Sets a string value.
    fn set_string(&self, namespace: &str, key: &str, value: &str) {
        self.set_custom_data(namespace, key, &string_tree(value));
    }

    /// Gets a string value.
    fn get_string(&self, namespace: &str, key: &str) -> Option<String> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::StringTag(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Sets a 32-bit integer value.
    fn set_int(&self, namespace: &str, key: &str, value: i32) {
        self.set_custom_data(namespace, key, &int_tree(value));
    }

    /// Gets a 32-bit integer value.
    fn get_int(&self, namespace: &str, key: &str) -> Option<i32> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Int(v) => Some(*v),
            NbtTag::Byte(v) => Some(i32::from(*v)),
            NbtTag::Short(v) => Some(i32::from(*v)),
            _ => None,
        }
    }

    /// Sets a 64-bit integer value.
    fn set_long(&self, namespace: &str, key: &str, value: i64) {
        self.set_custom_data(namespace, key, &long_tree(value));
    }

    /// Gets a 64-bit integer value.
    fn get_long(&self, namespace: &str, key: &str) -> Option<i64> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Long(v) => Some(*v),
            NbtTag::Int(v) => Some(i64::from(*v)),
            NbtTag::Byte(v) => Some(i64::from(*v)),
            NbtTag::Short(v) => Some(i64::from(*v)),
            _ => None,
        }
    }

    /// Sets a 16-bit integer value.
    fn set_short(&self, namespace: &str, key: &str, value: i16) {
        self.set_custom_data(namespace, key, &short_tree(value));
    }

    /// Gets a 16-bit integer value.
    fn get_short(&self, namespace: &str, key: &str) -> Option<i16> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Short(v) => Some(*v),
            NbtTag::Byte(v) => Some(i16::from(*v)),
            _ => None,
        }
    }

    /// Sets an 8-bit byte value.
    fn set_byte(&self, namespace: &str, key: &str, value: i8) {
        self.set_custom_data(namespace, key, &byte_tree(value));
    }

    /// Gets an 8-bit byte value.
    fn get_byte(&self, namespace: &str, key: &str) -> Option<i8> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Byte(v) => Some(*v),
            _ => None,
        }
    }

    /// Sets a boolean value (stored as an NBT byte: 1 or 0).
    fn set_bool(&self, namespace: &str, key: &str, value: bool) {
        self.set_custom_data(namespace, key, &bool_tree(value));
    }

    /// Gets a boolean value.
    fn get_bool(&self, namespace: &str, key: &str) -> Option<bool> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Byte(v) => Some(*v != 0),
            _ => None,
        }
    }

    /// Sets a 32-bit float value.
    fn set_float(&self, namespace: &str, key: &str, value: f32) {
        self.set_custom_data(namespace, key, &float_tree(value));
    }

    /// Gets a 32-bit float value.
    fn get_float(&self, namespace: &str, key: &str) -> Option<f32> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Sets a 64-bit double value.
    fn set_double(&self, namespace: &str, key: &str, value: f64) {
        self.set_custom_data(namespace, key, &double_tree(value));
    }

    /// Gets a 64-bit double value.
    fn get_double(&self, namespace: &str, key: &str) -> Option<f64> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::Double(v) => Some(*v),
            NbtTag::Float(v) => Some(f64::from(*v)),
            _ => None,
        }
    }

    /// Sets a byte array value.
    fn set_byte_array(&self, namespace: &str, key: &str, value: Vec<i8>) {
        self.set_custom_data(namespace, key, &byte_array_tree(value));
    }

    /// Gets a byte array value.
    fn get_byte_array(&self, namespace: &str, key: &str) -> Option<Vec<i8>> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::ByteArray(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Sets an integer array value.
    fn set_int_array(&self, namespace: &str, key: &str, value: Vec<i32>) {
        self.set_custom_data(namespace, key, &int_array_tree(value));
    }

    /// Gets an integer array value.
    fn get_int_array(&self, namespace: &str, key: &str) -> Option<Vec<i32>> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::IntArray(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Sets a long array value.
    fn set_long_array(&self, namespace: &str, key: &str, value: Vec<i64>) {
        self.set_custom_data(namespace, key, &long_array_tree(value));
    }

    /// Gets a long array value.
    fn get_long_array(&self, namespace: &str, key: &str) -> Option<Vec<i64>> {
        let tree = self.get_custom_data(namespace, key)?;
        match tree.tags.get(tree.root as usize)? {
            NbtTag::LongArray(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl PersistentDataHolder for ItemStack {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.has_custom_data(namespace, key)
    }
}

impl PersistentDataHolder for Entity {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.has_custom_data(namespace, key)
    }
}

impl PersistentDataHolder for BlockEntity {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.has_custom_data(namespace, key)
    }
}

impl PersistentDataHolder for Chunk {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.has_custom_data(namespace, key)
    }
}

impl PersistentDataHolder for World {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.has_custom_data(namespace, key)
    }
}

impl PersistentDataHolder for Player {
    fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
        self.as_entity().set_custom_data(namespace, key, value);
    }

    fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
        self.as_entity().get_custom_data(namespace, key)
    }

    fn remove_custom_data(&self, namespace: &str, key: &str) {
        self.as_entity().remove_custom_data(namespace, key);
    }

    fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.as_entity().has_custom_data(namespace, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockHolder {
        data: RefCell<HashMap<(String, String), NbtTree>>,
    }

    impl MockHolder {
        fn new() -> Self {
            Self {
                data: RefCell::new(HashMap::new()),
            }
        }
    }

    impl PersistentDataHolder for MockHolder {
        fn set_custom_data(&self, namespace: &str, key: &str, value: &NbtTree) {
            self.data
                .borrow_mut()
                .insert((namespace.to_string(), key.to_string()), value.clone());
        }

        fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTree> {
            self.data
                .borrow()
                .get(&(namespace.to_string(), key.to_string()))
                .cloned()
        }

        fn remove_custom_data(&self, namespace: &str, key: &str) {
            self.data
                .borrow_mut()
                .remove(&(namespace.to_string(), key.to_string()));
        }

        fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
            self.data
                .borrow()
                .contains_key(&(namespace.to_string(), key.to_string()))
        }
    }

    #[test]
    fn persistent_data_typed_methods() {
        let holder = MockHolder::new();

        // String
        holder.set_string("my_mod", "greeting", "hello world");
        assert!(holder.has_custom_data("my_mod", "greeting"));
        assert_eq!(
            holder.get_string("my_mod", "greeting"),
            Some("hello world".to_string())
        );

        // Int
        holder.set_int("my_mod", "score", 9001);
        assert_eq!(holder.get_int("my_mod", "score"), Some(9001));

        // Long
        holder.set_long("my_mod", "large_id", 123_456_789_012);
        assert_eq!(holder.get_long("my_mod", "large_id"), Some(123_456_789_012));

        // Bool
        holder.set_bool("my_mod", "is_admin", true);
        assert_eq!(holder.get_bool("my_mod", "is_admin"), Some(true));

        // Float & Double
        holder.set_float("my_mod", "multiplier", 1.5);
        assert_eq!(holder.get_float("my_mod", "multiplier"), Some(1.5));

        holder.set_double("my_mod", "precise", std::f64::consts::PI);
        assert_eq!(
            holder.get_double("my_mod", "precise"),
            Some(std::f64::consts::PI)
        );

        // Byte array
        holder.set_byte_array("my_mod", "raw_bytes", vec![1, 2, 3, 4]);
        assert_eq!(
            holder.get_byte_array("my_mod", "raw_bytes"),
            Some(vec![1, 2, 3, 4])
        );

        // Remove
        holder.remove_custom_data("my_mod", "greeting");
        assert!(!holder.has_custom_data("my_mod", "greeting"));
        assert_eq!(holder.get_string("my_mod", "greeting"), None);
    }
}
