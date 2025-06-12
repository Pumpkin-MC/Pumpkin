pub mod context;
pub mod events;

use async_trait::async_trait;
pub use context::*;
pub use events::*;

/// Struct representing metadata for a plugin.
///
/// This struct contains essential information about a plugin, including its name,
/// version, authors, and a description. It is generic over a lifetime `'s` to allow
/// for string slices that are valid for the lifetime of the plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginMetadata<'s> {
    /// The name of the plugin.
    pub name: &'s str,
    /// The version of the plugin.
    pub version: &'s str,
    /// The authors of the plugin.
    pub authors: &'s str,
    /// A description of the plugin.
    pub description: &'s str,
}

/// Trait representing a plugin with asynchronous lifecycle methods.
///
/// This trait defines the required methods for a plugin, including hooks for when
/// the plugin is loaded and unloaded. It is marked with `async_trait` to allow
/// for asynchronous implementations.
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Asynchronous method called when the plugin is loaded.
    ///
    /// This method initializes the plugin within the server context.
    ///
    /// # Parameters
    /// - `_server`: Reference to the server's context.
    ///
    /// # Returns
    /// - `Ok(())` on success, or `Err(String)` on failure.
    async fn on_load(&mut self, _server: &Context) -> Result<(), String> {
        Ok(())
    }

    /// Asynchronous method called when the plugin is unloaded.
    ///
    /// This method cleans up resources when the plugin is removed from the server context.
    ///
    /// # Parameters
    /// - `_server`: Reference to the server's context.
    ///
    /// # Returns
    /// - `Ok(())` on success, or `Err(String)` on failure.
    async fn on_unload(&mut self, _server: &Context) -> Result<(), String> {
        Ok(())
    }
}

/// The `NamespacedKey` struct
#[allow(dead_code)]
pub struct NamespacedKey {
    namespace: String,
    key: String,
}

#[derive(Debug)]
pub enum NamespacedKeyError {
    NonAsciiNamespace,
    NonAsciiKey,
}

/// The `NamespacedKey` constructor
///
/// `new()` must only be called via the `ns_key!` macro.
///
/// # Parameters
/// - `namespace`: namespace of the key, must be equal to the `CARGO_PKG_NAME`
/// - `key`: The key as a String
///
/// # Returns
/// - Self
impl NamespacedKey {
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn new(namespace: &str, key: &str) -> Result<Self, NamespacedKeyError> {
        if !namespace.is_ascii() {
            return Err(NamespacedKeyError::NonAsciiNamespace);
        }
        if !key.is_ascii() {
            return Err(NamespacedKeyError::NonAsciiKey);
        }

        Ok(Self {
            namespace: namespace.to_ascii_lowercase(),
            key: key.to_ascii_lowercase(),
        })
    }
}

/// A macro used to create a new `NamespacedKey` without having to manually pass the `CARGO_PKG_NAME` by using the `env!()` macro
///
/// # Parameters
/// - `$value`: the key you want to create as a String.
#[macro_export]
macro_rules! ns_key {
    ($value:expr) => {
        match $crate::plugin::NamespacedKey::new(env!("CARGO_PKG_NAME"), $value) {
            Ok(key) => key,
            Err(_) => panic!("Invalid key: must be pure ASCII"),
        }
    };
}

// Tests
#[cfg(test)]
mod tests {
    use crate::ns_key;
    use crate::plugin::NamespacedKey;
    use crate::plugin::NamespacedKeyError;

    #[test]
    fn test_rejects_unicode_key() {
        let result = NamespacedKey::new("myplugin", "Klöten");
        assert!(matches!(result, Err(NamespacedKeyError::NonAsciiKey)));
    }

    #[test]
    fn test_accepts_ascii_key() {
        let result = NamespacedKey::new("myplugin", "valid_key");
        assert!(result.is_ok());
    }

    #[test]
    fn test_macro_lowercase() {
        let expected_namespace = env!("CARGO_PKG_NAME").to_ascii_lowercase();

        let key = ns_key!("MyKey");

        assert_eq!(key.namespace, expected_namespace);
        assert_eq!(key.key, "mykey");
    }

    #[test]
    fn test_macro_uppercase() {
        let expected_namespace = env!("CARGO_PKG_NAME").to_ascii_lowercase();

        let key = ns_key!("UpperCASEKey");

        assert_eq!(key.namespace, expected_namespace);
        assert_eq!(key.key, "uppercasekey");
    }

    #[test]
    fn test_macro_is_deterministic() {
        let a = ns_key!("SomeKey");
        let b = ns_key!("SomeKey");

        assert_eq!(a.namespace, b.namespace);
        assert_eq!(a.key, b.key);
    }

    #[test]
    fn test_macro_key_inequality() {
        let a = ns_key!("KeyA");
        let b = ns_key!("KeyB");

        assert_ne!(a.key, b.key);
    }
}
