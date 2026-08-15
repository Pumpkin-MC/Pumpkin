//! Template caching for embedded structure templates.
//!
//! This module provides a lazy-loading cache for structure templates that are
//! embedded in the binary at compile time using `include_bytes!`.
//! Datapack structures in `data/<namespace>/structure/*.nbt` are loaded on
//! demand from the filesystem when not found in the embedded data.

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use std::sync::RwLock;

use super::{StructureTemplate, structure_template::TemplateError};

/// A cache for loaded structure templates.
///
/// Templates are loaded lazily on first access and stored for reuse.
/// The cache is thread-safe and can be accessed from multiple threads.
pub struct TemplateCache {
    cache: DashMap<String, Arc<StructureTemplate>>,
    /// Optional paths to search for datapack structure .nbt files.
    datapack_search_paths: RwLock<Vec<PathBuf>>,
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
            datapack_search_paths: RwLock::new(Vec::new()),
        }
    }

    /// Set search paths for datapack structure files.
    pub fn set_datapack_paths(&self, paths: Vec<PathBuf>) {
        *self
            .datapack_search_paths
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = paths;
    }

    /// Try to load a structure .nbt file from datapack search paths.
    fn load_datapack_bytes(&self, name: &str) -> Option<Vec<u8>> {
        let paths = self
            .datapack_search_paths
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for base in paths.iter() {
            // Try: <base>/data/<namespace>/structure/<name>.nbt
            // Handle both "namespace:path" and bare "path" forms
            let (namespace, rel_path) = name.split_once(':').unwrap_or(("minecraft", name));
            let rel_path = rel_path.trim_start_matches('/');
            let namespaces = vec![
                format!("{namespace}/structures/{rel_path}"),
                format!("{namespace}/structure/{rel_path}"),
            ];
            for ns_path in &namespaces {
                let file_path = base.join("data").join(ns_path);
                let with_ext = if file_path.extension().is_none() {
                    file_path.with_extension("nbt")
                } else {
                    file_path
                };
                if let Ok(data) = std::fs::read(&with_ext) {
                    return Some(data);
                }
                // Also try with .nbt extension appended
                let alt = format!("{}.nbt", with_ext.display());
                if let Ok(data) = std::fs::read(&alt) {
                    return Some(data);
                }
            }
        }
        None
    }

    /// Gets a template by `name`, loading from embedded resources or datapack files.
    ///
    /// Returns the loaded template wrapped in an `Arc`, or `None` if the template
    /// doesn't exist or failed to load.
    pub fn get(&self, name: &str) -> Option<Arc<StructureTemplate>> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);

        // Check cache first
        if let Some(template) = self.cache.get(name) {
            return Some(Arc::clone(&template));
        }

        // Try embedded bytes first
        if let Some(bytes) = Self::load_template_bytes(name)
            && let Ok(template) = StructureTemplate::from_nbt_bytes(bytes)
        {
            let arc = Arc::new(template);
            self.cache.insert(name.to_owned(), Arc::clone(&arc));
            return Some(arc);
        }

        // Fall back to datapack filesystem paths
        if let Some(bytes) = self.load_datapack_bytes(name)
            && let Ok(template) = StructureTemplate::from_nbt_bytes(&bytes)
        {
            let arc = Arc::new(template);
            self.cache.insert(name.to_owned(), Arc::clone(&arc));
            return Some(arc);
        }

        None
    }

    /// Gets a template by name, returning an error if loading fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the template doesn't exist or fails to parse.
    pub fn get_or_error(&self, name: &str) -> Result<Arc<StructureTemplate>, TemplateError> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);

        // Check cache first
        if let Some(template) = self.cache.get(name) {
            return Ok(Arc::clone(&template));
        }

        // Try embedded bytes first
        if let Some(bytes) = Self::load_template_bytes(name) {
            let template = StructureTemplate::from_nbt_bytes(bytes)?;
            let arc = Arc::new(template);
            self.cache.insert(name.to_owned(), Arc::clone(&arc));
            return Ok(arc);
        }

        // Fall back to datapack filesystem paths
        if let Some(bytes) = self.load_datapack_bytes(name) {
            let template = StructureTemplate::from_nbt_bytes(&bytes)?;
            let arc = Arc::new(template);
            self.cache.insert(name.to_owned(), Arc::clone(&arc));
            return Ok(arc);
        }

        Err(TemplateError::MissingField("template file not found"))
    }

    /// Preloads a list of templates into the cache.
    ///
    /// This can be useful during server startup to avoid loading delays
    /// during gameplay.
    pub fn preload(&self, names: &[&'static str]) {
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
    ///
    /// This function maps template names to their embedded byte data.
    /// Add new templates here as they are added to the assets.
    fn load_template_bytes(path: &str) -> Option<&'static [u8]> {
        get_template_bytes(path)
    }
}

include!(concat!(env!("OUT_DIR"), "/template_embeddings.rs"));

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
/// Useful for tab-completion in commands.
#[must_use]
#[allow(clippy::used_underscore_items)]
pub const fn all_template_names() -> &'static [&'static str] {
    _generated_all_template_names()
}

/// Returns a list of all available structure names for `/place structure` tab-completion.
#[must_use]
pub const fn all_structure_names() -> &'static [&'static str] {
    pumpkin_data::structures::StructureKeys::all_names()
}

/// Returns a list of all available pool names for `/place jigsaw` tab-completion.
#[must_use]
#[allow(clippy::used_underscore_items)]
pub const fn all_pool_names() -> &'static [&'static str] {
    _generated_all_pool_names()
}
