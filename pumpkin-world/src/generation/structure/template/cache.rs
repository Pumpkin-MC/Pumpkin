//! Template caching for embedded structure templates.
//!
//! This module provides a lazy-loading cache for structure templates that are
//! embedded in the binary at compile time using `include_bytes!`.

use std::sync::Arc;

use dashmap::DashMap;

use super::{StructureTemplate, structure_template::TemplateError};

/// A cache for loaded structure templates.
///
/// Templates are loaded lazily on first access and stored for reuse.
/// The cache is thread-safe and can be accessed from multiple threads.
pub struct TemplateCache {
    cache: DashMap<&'static str, Arc<StructureTemplate>>,
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
        }
    }

    /// Gets a template by `name`, loading it from embedded resources if not cached.
    ///
    /// Returns the loaded template wrapped in an `Arc`, or `None` if the template
    /// doesn't exist or failed to load.
    pub fn get(&self, name: &'static str) -> Option<Arc<StructureTemplate>> {
        // Check cache first
        if let Some(template) = self.cache.get(name) {
            return Some(Arc::clone(&template));
        }

        // Try to load the template
        let bytes = Self::load_template_bytes(name)?;

        match StructureTemplate::from_nbt_bytes(bytes) {
            Ok(template) => {
                let arc = Arc::new(template);
                self.cache.insert(name, Arc::clone(&arc));
                Some(arc)
            }
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
    pub fn get_or_error(&self, name: &'static str) -> Result<Arc<StructureTemplate>, TemplateError> {
        // Check cache first
        if let Some(template) = self.cache.get(name) {
            return Ok(Arc::clone(&template));
        }

        // Try to load the template
        let bytes = Self::load_template_bytes(name)
            .ok_or(TemplateError::MissingField("template file not found"))?;

        let template = StructureTemplate::from_nbt_bytes(bytes)?;
        let arc = Arc::new(template);
        self.cache.insert(name, Arc::clone(&arc));
        Ok(arc)
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
        // Map template names to embedded files
        // Templates are stored in pumpkin-world/assets/structures/
        match path {
            // Igloo templates
            "igloo/top" => Some(include_bytes!("../../../../assets/structures/igloo/top.nbt")),
            "igloo/middle" => Some(include_bytes!("../../../../assets/structures/igloo/middle.nbt")),
            "igloo/bottom" => Some(include_bytes!("../../../../assets/structures/igloo/bottom.nbt")),

            // Add more templates here as needed:
            // "shipwreck/with_mast" => Some(include_bytes!("../../../assets/structures/shipwreck/with_mast.nbt")),
            // "village/plains/houses/small_house_1" => Some(include_bytes!("../../../assets/structures/village/plains/houses/small_house_1.nbt")),
            _ => None,
        }
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
pub fn get_template(name: &'static str) -> Option<Arc<StructureTemplate>> {
    global_cache().get(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = TemplateCache::new();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_unknown_template_returns_none() {
        let cache = TemplateCache::new();
        assert!(cache.get("nonexistent/template").is_none());
    }

    #[test]
    fn test_load_igloo_top() {
        let cache = TemplateCache::new();
        let template = cache.get("igloo/top").expect("Failed to load igloo/top");

        // Verify vanilla igloo/top dimensions (7x5x8)
        assert_eq!(template.size.x, 7);
        assert_eq!(template.size.y, 5);
        assert_eq!(template.size.z, 8);

        // Verify it has blocks
        assert!(!template.blocks.is_empty());
        // Verify it has a palette
        assert!(!template.palette.is_empty());
    }

    #[test]
    fn test_load_igloo_middle() {
        let cache = TemplateCache::new();
        let template = cache.get("igloo/middle").expect("Failed to load igloo/middle");

        // Verify vanilla igloo/middle dimensions (3x3x3)
        assert_eq!(template.size.x, 3);
        assert_eq!(template.size.y, 3);
        assert_eq!(template.size.z, 3);
    }

    #[test]
    fn test_load_igloo_bottom() {
        let cache = TemplateCache::new();
        let template = cache.get("igloo/bottom").expect("Failed to load igloo/bottom");

        // Verify vanilla igloo/bottom dimensions (7x6x9)
        assert_eq!(template.size.x, 7);
        assert_eq!(template.size.y, 6);
        assert_eq!(template.size.z, 9);
    }

    #[test]
    fn test_cache_reuses_templates() {
        let cache = TemplateCache::new();

        // Load twice
        let first = cache.get("igloo/top").expect("First load failed");
        let second = cache.get("igloo/top").expect("Second load failed");

        // Should be the same Arc (pointer equality)
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.len(), 1);
    }
}
