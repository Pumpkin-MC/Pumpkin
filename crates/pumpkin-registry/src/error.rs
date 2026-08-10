use std::any::TypeId;

use pumpkin_util::identifier::Identifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataKeyBuildError {
    #[error("a data key must contain at least one identifier")]
    Empty,

    #[error("registry `{0}` does not exist")]
    MissingRegistry(Identifier),

    #[error("entry `{0}` is not a nested registry")]
    NotARegistry(Identifier),

    #[error("value `{0}` does not exist")]
    MissingValue(Identifier),

    #[error("registry entry type mismatch: expected `{expected}`, found `{actual}`")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum DataKeyGetError {
    #[error("the data key contains no IDs")]
    InvalidKey,

    #[error("nested registry with numeric ID {id} does not exist")]
    MissingRegistry { id: usize },

    #[error("value with numeric ID {id} does not exist")]
    MissingValue { id: usize },

    #[error("registry entry type mismatch: expected `{expected}`, found `{actual}`")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("bootstrap provider for registry {registry} returned the wrong entry type")]
    TypeMismatch {
        registry: Identifier,
        expected: TypeId,
        actual: TypeId,
    },

    #[error("duplicate entry `{identifier}` in registry `{registry}`")]
    DuplicateEntry {
        registry: Identifier,
        identifier: Identifier,
    },
}
