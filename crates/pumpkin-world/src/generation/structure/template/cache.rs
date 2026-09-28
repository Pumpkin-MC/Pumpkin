//! Template caching for embedded structure templates.
//!
//! This module provides a lazy-loading cache for structure templates that are
//! embedded in the binary at compile time using `include_bytes!`.

use std::sync::Arc;

use dashmap::DashMap;

use super::{StructureTemplate, structure_template::TemplateError};

/// Vanilla's implicit namespace.
const DEFAULT_NAMESPACE: &str = "minecraft";

/// Canonicalizes a resource id to fully-qualified `namespace:path` form.
///
/// A bare `foo` becomes `minecraft:foo`, matching vanilla resolution.
fn canonicalize(name: &str) -> String {
    if name.contains(':') {
        name.to_owned()
    } else {
        format!("{DEFAULT_NAMESPACE}:{name}")
    }
}

/// A cache for loaded structure templates.
///
/// Templates are loaded lazily on first access and stored for reuse.
/// Keys are fully-qualified resource ids, so `foo` and `minecraft:foo`
/// share a single entry.
/// The cache is thread-safe and can be accessed from multiple threads.
pub struct TemplateCache {
    cache: DashMap<String, Arc<StructureTemplate>>,
    dynamic_templates: DashMap<String, Arc<[u8]>>,
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateCache {
    /// Creates a new empty template cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            dynamic_templates: DashMap::new(),
        }
    }

    /// Registers raw template NBT bytes from a dynamic datapack.
    pub fn register_template(&self, name: &str, bytes: Arc<[u8]>) {
        let key = canonicalize(name);
        self.dynamic_templates.insert(key.clone(), bytes);
        self.cache.remove(&key);
    }

    /// Clears all dynamically registered templates (e.g. during datapack reload).
    pub fn clear_dynamic(&self) {
        self.dynamic_templates.clear();
        self.cache.clear();
    }

    /// Gets a template by `name`, loading it from embedded resources if not cached.
    ///
    /// `name` may be bare (`foo`) or namespaced (`minecraft:foo`, `pumpkin:foo`).
    ///
    /// Returns the loaded template wrapped in an `Arc`, or `None` if the template
    /// doesn't exist or failed to load.
    pub fn get(&self, name: &str) -> Option<Arc<StructureTemplate>> {
        match self.get_or_error(name) {
            Ok(template) => Some(template),
            Err(TemplateError::MissingField("template file not found")) => None,
            Err(e) => {
                tracing::error!("Failed to load template '{}': {}", name, e);
                None
            }
        }
    }

    /// Gets a template by name, returning an error if loading fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the template doesn't exist or fails to parse.
    pub fn get_or_error(&self, name: &str) -> Result<Arc<StructureTemplate>, TemplateError> {
        let key = canonicalize(name);

        // Check cache first
        if let Some(template) = self.cache.get(&key) {
            return Ok(Arc::clone(&template));
        }

        let mut template = if let Some(bytes) = self.dynamic_templates.get(&key) {
            StructureTemplate::from_nbt_bytes(&bytes)?
        } else if let Some(bytes) = Self::load_template_bytes(&key) {
            StructureTemplate::from_nbt_bytes(bytes)?
        } else {
            return Err(TemplateError::MissingField("template file not found"));
        };
        template.name = Some(key.clone());

        let arc = Arc::new(template);
        self.cache.insert(key, Arc::clone(&arc));
        Ok(arc)
    }

    /// Preloads a list of templates into the cache.
    ///
    /// This can be useful during server startup to avoid loading delays
    /// during gameplay.
    pub fn preload(&self, names: &[&str]) {
        for name in names {
            if let Err(e) = self.get_or_error(name) {
                tracing::warn!("Failed to preload template '{}': {}", name, e);
            }
        }
    }

    /// Returns the number of cached templates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clears all cached templates.
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Loads raw template bytes from embedded resources.
    fn load_template_bytes(path: &str) -> Option<&'static [u8]> {
        pumpkin_data::template_bytes::get_template_bytes(path)
    }
}

/// Global template cache instance.
///
/// This provides a singleton cache that can be used throughout the codebase
/// without needing to pass around a cache reference.
static GLOBAL_CACHE: std::sync::LazyLock<TemplateCache> =
    std::sync::LazyLock::new(TemplateCache::new);

/// Gets the global template cache.
#[must_use]
pub fn global_cache() -> &'static TemplateCache {
    &GLOBAL_CACHE
}

/// Gets a template by `name` from the global cache.
///
/// Returns the loaded template wrapped in an `Arc`, or `None` if not found.
#[must_use]
pub fn get_template(name: &str) -> Option<Arc<StructureTemplate>> {
    global_cache().get(name)
}

/// Returns a list of all available template names that can be loaded.
///
/// These are derived from the embedded structure files at compile time.
/// Names are fully qualified (e.g. `minecraft:village/plains/houses/...`).
/// Useful for tab-completion in commands.
#[must_use]
pub const fn all_template_names() -> &'static [&'static str] {
    pumpkin_data::template_bytes::all_template_names()
}

/// Returns a list of all available structure names for `/place structure` tab-completion.
#[must_use]
pub const fn all_structure_names() -> &'static [&'static str] {
    pumpkin_data::structures::StructureKeys::all_names()
}

/// Returns a list of all available pool names for `/place jigsaw` tab-completion.
#[must_use]
pub const fn all_pool_names() -> &'static [&'static str] {
    pumpkin_data::template_pool::StaticTemplatePool::all_names()
}

/// Returns raw NBT bytes for an embedded structure template.
#[must_use]
pub fn template_bytes(name: &str) -> Option<&'static [u8]> {
    pumpkin_data::template_bytes::get_template_bytes(&canonicalize(name))
}

#[must_use]
pub const fn all_embedded_datapack_names() -> &'static [&'static str] {
    pumpkin_data::template_bytes::all_embedded_datapack_names()
}
