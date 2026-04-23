use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::world::LevelConfig;

/// Selects the backend used for persistent server state.
///
/// `Vanilla` wraps a [`LevelConfig`] carrying the file-backed layout's
/// per-world settings (chunk format, lighting, autosave). `InMemory` is
/// ephemeral and has no vanilla-specific options, so it carries nothing.
///
/// Fields needed by both backends (lighting, autosave cadence) are read
/// through [`StorageConfig::level`], which returns a default [`LevelConfig`]
/// for `InMemory`.
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum StorageConfig {
    #[serde(rename = "vanilla")]
    Vanilla(LevelConfig),
    #[serde(rename = "in_memory")]
    InMemory,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::Vanilla(LevelConfig::default())
    }
}

impl StorageConfig {
    /// Returns the level-wide settings regardless of backend. In-memory
    /// storage has no file-layout options, so a defaulted [`LevelConfig`]
    /// is synthesised for those callers.
    #[must_use]
    pub fn level(&self) -> Cow<'_, LevelConfig> {
        match self {
            Self::Vanilla(cfg) => Cow::Borrowed(cfg),
            Self::InMemory => Cow::Owned(LevelConfig::default()),
        }
    }
}
