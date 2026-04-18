use std::path::PathBuf;

/// Errors reported by all domain-specific storage traits in this crate.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("not found: {message}")]
    NotFound { message: String },

    #[error("unsupported on-disk version: {0}")]
    UnsupportedVersion(String),

    #[error("deserialization failed: {0}")]
    Deserialize(String),

    #[error("serialization failed: {0}")]
    Serialize(String),

    #[error("io error at {path:?}: {source}")]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },
}

impl StorageError {
    /// `true` when the error means "no such resource".
    ///
    /// Accepts both the explicit [`StorageError::NotFound`] variant and the
    /// `ErrorKind::NotFound` case of an I/O error — filesystem backends often
    /// surface missing-path errors through `std::io::Error`.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::NotFound { .. } => true,
            Self::Io { source, .. } => source.kind() == std::io::ErrorKind::NotFound,
            _ => false,
        }
    }

    /// Wraps an I/O error with no associated path.
    pub fn io(source: std::io::Error) -> Self {
        Self::Io {
            path: None,
            source,
        }
    }

    /// Wraps an I/O error tagged with the path that caused it.
    pub fn io_at(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: Some(path.into()),
            source,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(source: std::io::Error) -> Self {
        Self::io(source)
    }
}
