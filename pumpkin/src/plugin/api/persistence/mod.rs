pub mod nbt;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Represents a key with an associated namespace.
///
/// This struct is used to differentiate persistent data by plugin through namespacing.
#[allow(dead_code)]
#[derive(Eq, Hash, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct NamespacedKey {
    pub(crate) namespace: String,
    pub(crate) key: String,
}

#[derive(Debug)]
pub enum NamespacedKeyError {
    NonAsciiNamespace,
    NonAsciiKey,
    AmbiguousKey,
}

/// Constructs a new `NamespacedKey`.
///
/// Note: This method should only be called via the `ns_key!` macro.
///
/// # Parameters
/// - `namespace`: The namespace for the key, typically set to the crate's name (`CARGO_PKG_NAME`).
/// - `key`: The key string.
///
/// # Returns
/// - `Ok(Self)` if both namespace and key contain only ASCII characters (case-insensitive).
/// - `Err(NamespacedKeyError)` if either contains non-ASCII characters.
impl NamespacedKey {
    #[allow(dead_code)]
    pub fn new(namespace: &str, key: &str) -> Result<Self, NamespacedKeyError> {
        if !namespace.is_ascii() {
            #[cfg(debug_assertions)]
            log::error!("Invalid namespace: '{namespace}' contains non-ASCII characters.");
            return Err(NamespacedKeyError::NonAsciiNamespace);
        }
        if !key.is_ascii() {
            #[cfg(debug_assertions)]
            log::error!("Invalid key: '{key}' contains non-ASCII characters.");
            return Err(NamespacedKeyError::NonAsciiKey);
        }

        Ok(Self {
            namespace: namespace.to_ascii_lowercase(),
            key: key.to_ascii_lowercase(),
        })
    }
}

impl std::fmt::Display for NamespacedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.key)
    }
}

/// Macro to conveniently create a `NamespacedKey` using the crate's package name as the namespace.
///
/// # Parameters
/// - `$value`: The key string.
///
/// # Panics
/// Panics if the key or namespace contains non-ASCII characters.
#[macro_export]
macro_rules! ns_key {
    ($value:expr) => {
        match ::pumpkin::plugin::persistence::NamespacedKey::new(env!("CARGO_PKG_NAME"), $value) {
            Ok(key) => key,
            Err(e) => panic!("ns_key! macro failed: {:?}", e),
        }
    };
}

/// Type alias for a concurrent map that holds `NamespacedKey`s associated with their persistent values.
///
/// This container uses `DashMap` to provide thread-safe concurrent access.
///
/// Note: This is `pub(crate)` so not all methods on `DashMap` are needlessly exposed.
/// Instead, the methods from the `PersistentDataHolder` trait should be used.
pub(crate) type PersistentDataContainer = DashMap<NamespacedKey, PersistentDataType>;

/// Enum representing all allowed data types that can be stored in a `PersistentDataContainer`.
#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum PersistentDataType {
    Bool(bool),
    String(String),
    Char(char),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Box<[u8]>),
    List(Vec<PersistentDataType>),
}

/// Trait defining common operations for structs that hold a `PersistentDataContainer`.
///
/// This trait is typically implemented via the `PersistentDataHolder` derive macro,
/// which auto-generates the necessary method implementations.
pub trait PersistentDataHolder {
    /// Clears all stored data.
    fn clear(&self);
    /// Retrieves the value associated with the specified key, if it exists.
    fn get(&self, key: &NamespacedKey) -> Option<PersistentDataType>;
    /// Retrieves the value associated with the specified key, converted into type `T` if possible.
    fn get_as<T: FromPersistentDataType>(&self, key: &NamespacedKey) -> Option<T>;
    /// Inserts or updates the value for the specified key.
    fn insert(&self, key: &NamespacedKey, value: PersistentDataType);
    /// Removes the value associated with the specified key and returns it if present.
    fn remove(&self, key: &NamespacedKey) -> Option<PersistentDataType>;
    /// Checks whether the container contains the specified key.
    fn contains_key(&self, key: &NamespacedKey) -> bool;
    /// Returns an iterator over all key-value pairs in the container.
    fn iter(&self) -> Box<dyn Iterator<Item = (NamespacedKey, PersistentDataType)> + '_>;
    /// Returns a mutable reference of the container
    fn container_mut(&mut self) -> &mut PersistentDataContainer;
}

/// Trait to extract the inner value from a `PersistentDataType`.
///
/// This trait enables type-safe conversion from the enum wrapper to the contained value.
pub trait FromPersistentDataType: Sized {
    /// Attempts to convert a reference to a `PersistentDataType` into the implementing type.
    fn from_persistent(value: &PersistentDataType) -> Option<Self>;
}

/// Macro to simplify implementation of `FromPersistentDataType` for types with repetitive logic.
///
/// Supports both `Copy` and `Clone` types.
///
/// # Usage
/// - For `Copy` types: `from_persistent!(VariantName, Type);`
/// - For `Clone` types: `from_persistent!(clone VariantName, Type);`
#[macro_export]
macro_rules! from_persistent {
    // For Copy types
    ($variant:ident, $ty:ty) => {
        impl FromPersistentDataType for $ty {
            fn from_persistent(value: &PersistentDataType) -> Option<Self> {
                match value {
                    PersistentDataType::$variant(v) => Some(*v),
                    _ => None,
                }
            }
        }
    };

    // For Clone types
    (clone $variant:ident, $ty:ty) => {
        impl FromPersistentDataType for $ty {
            fn from_persistent(value: &PersistentDataType) -> Option<Self> {
                match value {
                    PersistentDataType::$variant(v) => Some(v.clone()),
                    _ => None,
                }
            }
        }
    };
}

// Copy types
from_persistent!(Bool, bool);
from_persistent!(Char, char);
from_persistent!(I32, i32);
from_persistent!(I64, i64);
from_persistent!(U8, u8);
from_persistent!(U16, u16);
from_persistent!(U32, u32);
from_persistent!(U64, u64);
from_persistent!(F32, f32);
from_persistent!(F64, f64);

// Clone types
from_persistent!(clone String, String);
from_persistent!(clone Bytes, Box<[u8]>);
from_persistent!(clone List, Vec<PersistentDataType>);
