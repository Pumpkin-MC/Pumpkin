use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::plugin::NamespacedKey;

/// The supported persistent data types.
#[derive(PartialEq, Debug)]
pub enum PersistentValue {
   Bool(bool),
   String(String),
   I32(i32),
   I64(i64),
   U32(u32),
   U64(u64),
   F32(f32),
   F64(f64),
   Bytes(Vec<u8>),
   List(Vec<PersistentValue>),
}

/// The `PersistentDataContainer` struct
///
/// This struct contains `NamespacedKey`s and associates them with `PersistentValue`s using a `HashMap`.
#[derive(Default)]
pub struct PersistentDataContainer {
   pub data: Arc<Mutex<HashMap<NamespacedKey, PersistentValue>>>,
}

impl PersistentDataContainer {
   /// Creates a new, empty `PersistentDataContainer`.
   ///
   /// This initializes the internal storage as an empty `HashMap`
   /// wrapped in an `Arc<Mutex<...>>` for thread-safe, shared access.
   ///
   /// # Returns
   /// A new instance of `PersistentDataContainer`.
   pub fn new() -> Self {
      Self {
         data: Arc::new(Mutex::new(HashMap::new())),
      }
   }

   /// Clears all stored key-value pairs in the container.
   ///
   /// This method acquires a lock on the internal `HashMap`
   /// and removes all entries.
   ///
   /// # Panics
   /// This method panics if the mutex is poisoned.
   pub fn clear(&self) {
      let mut map = self.data.lock().unwrap();
      map.clear();
   }

   /// Saves a value under the given `NamespacedKey`.
   ///
   /// If the key already exists, its value is overwritten.
   ///
   /// # Parameters
   /// - `key`: The unique key under which the value should be stored.
   /// - `value`: The `PersistentValue` to store.
   ///
   /// # Panics
   /// This method panics if the mutex is poisoned.
   pub fn save_data(&self, key: NamespacedKey, value: PersistentValue) {
      let mut map = self.data.lock().unwrap();
      map.insert(key, value);
   }

   /// Retrieves a stored value by its `NamespacedKey`.
   ///
   /// # Parameters
   /// - `key`: A reference to the `NamespacedKey` to look up.
   ///
   /// # Returns
   /// An `Option<PersistentValue>`: `Some(value)` if the key exists, `None` otherwise.
   ///
   /// # Panics
   /// This method panics if the mutex is poisoned.
   pub fn get_data(&self, key: &NamespacedKey) -> Option<PersistentValue> {
      let map = self.data.lock().unwrap();
      map.get(key).cloned()
   }
}

/// A generic wrapper that associates a `PersistentDataContainer` with any arbitrary struct.
///
/// In contrast to the familiar implementation of `PersistentDataContainer` in bukkit and its derivatives,
/// the `PersistentDataHolder` is not an interface to be implemented in a class. Rather, it itself 
/// holds the reference to an object.
/// 
/// This wrapper enables data persistence for any struct `T` by attaching a `PersistentDataContainer`
/// alongside it. It simplifies managing additional metadata or dynamic properties without modifying
/// the original struct.
///
/// # Type Parameters
/// - `T`: The type of the wrapped struct.
///
/// # Example
/// ```
/// let entity = MyStruct::new();
/// let holder = PersistentDataHolder::new(&entity);
///
/// let key = ns_key!("example_key");
/// holder.save_data(key, PersistentValue::String("value".to_string()));
/// ```
pub struct PersistentDataHolder<'a, T> {
   /// A reference to the wrapped inner struct.
   pub inner: &'a T,

   /// The optional persistent data container associated with the struct.
   pub container: Option<PersistentDataContainer>,
}

impl<'a, T> PersistentDataHolder<'a, T> {
   /// Creates a new `PersistentDataHolder` for a given struct reference.
   ///
   /// # Parameters
   /// - `inner`: A reference to the struct to associate with a container.
   ///
   /// # Returns
   /// A new instance of `PersistentDataHolder<T>`.
   pub fn new(inner: &'a T) -> Self {
      Self {
         inner,
         container: Some(PersistentDataContainer::new()),
      }
   }

   /// Retrieves a stored value from the container by key.
   ///
   /// # Parameters
   /// - `key`: A reference to the `NamespacedKey` to look up.
   ///
   /// # Returns
   /// An `Option<PersistentValue>` if the key exists.
   pub fn get_data(&self, key: &NamespacedKey) -> Option<PersistentValue> {
      self.container
         .as_ref()
         .and_then(|container| container.get_data(key))
   }

   /// Saves a value in the container under the specified key.
   ///
   /// # Parameters
   /// - `key`: The key under which to store the value.
   /// - `value`: The value to store.
   pub fn save_data(&self, key: NamespacedKey, value: PersistentValue) {
      if let Some(container) = &self.container {
         container.save_data(key, value);
      }
   }

   /// Removes a specific value that is linked to a given key.
   ///
   /// This method acquires a lock on the internal data container and removes the entry
   /// for the specified key, if it exists.
   ///
   /// # Parameters
   /// - `key`: The `NamespacedKey` to remove.
   ///
   /// # Panics
   /// This method will panic if the underlying mutex is poisoned.
   pub fn remove_by_key(&self, key: NamespacedKey) {
      let mut map = self.data.lock().unwrap();
      map.remove(&key);
   }

   /// Clears all entries from the associated data container.
   pub fn clear(&self) {
      if let Some(container) = &self.container {
         container.clear();
      }
   }

   /// Gets a mutable reference to the internal container wrapped in `Option`.
   ///
   /// This is used internally for operations like `destroy_container`.
   ///
   /// # Returns
   /// A mutable reference to the internal container wrapped in an `Option`.
   pub fn get_container_mut(&mut self) -> &mut Option<PersistentDataContainer> {
      &mut self.container
   }

   /// Destroys the container by setting it to `None`.
   ///
   /// After calling this, the holder will no longer contain any persistent data.
   pub fn destroy_container(&mut self) {
      self.container = None;
   }
}