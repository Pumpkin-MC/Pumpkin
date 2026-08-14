pub mod detector;
pub mod format;
pub mod metadata;
pub mod repository;
pub mod resource;
pub mod source;

use std::sync::Arc;

use format::PackCompatibility;
use metadata::PackMcmeta;
use resource::PackResources;

/// Where a pack came from (determines auto-enable behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSource {
    /// Built-in "vanilla" pack.
    BuiltIn,
    /// Pack from `world/datapacks/`.
    World,
    /// Plugin-provided pack.
    Plugin,
}

impl PackSource {
    /// Whether this pack should be automatically enabled when discovered.
    #[must_use]
    pub const fn should_add_automatically(self) -> bool {
        matches!(self, Self::World)
    }
}

/// A discovered datapack with its metadata and resource access.
#[derive(Clone)]
pub struct Pack {
    /// Unique identifier (e.g., "vanilla", "file/somepack").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Resource reader.
    pub resources: Option<Arc<dyn PackResources>>,
    /// Parsed pack.mcmeta.
    pub metadata: Option<Box<PackMcmeta>>,
    /// Source of the pack.
    pub source: PackSource,
    /// Version compatibility.
    pub compatibility: PackCompatibility,
    /// Required feature flags from pack.mcmeta.
    pub feature_flags: Vec<String>,
}

impl Pack {
    /// Create the built-in "vanilla" pack.
    #[must_use]
    pub fn vanilla() -> Self {
        Self {
            id: "vanilla".to_string(),
            name: "Vanilla".to_string(),
            resources: Some(Arc::new(resource::VanillaPackResources)),
            metadata: None,
            source: PackSource::BuiltIn,
            compatibility: PackCompatibility::Compatible,
            feature_flags: Vec::new(),
        }
    }
}
