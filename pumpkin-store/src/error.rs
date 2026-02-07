/// Errors from the store layer.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("block not found: {0}")]
    BlockNotFound(String),

    #[error("item not found: {0}")]
    ItemNotFound(String),

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[cfg(feature = "lance-store")]
    #[error("lance error: {0}")]
    Lance(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Convenience alias.
pub type StoreResult<T> = Result<T, StoreError>;
