use std::any::TypeId;

use pumpkin_util::identifier::Identifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataKeyGetError {
    #[error("data key is empty or invalid")]
    InvalidKey,

    #[error("identifier `{identifier}` does not exist")]
    MissingIdentifier {
        identifier: Identifier,
    },

    #[error("registry with id {id} does not exist")]
    MissingRegistry {
        id: usize,
    },

    #[error("value with id {id} does not exist")]
    MissingValue {
        id: usize,
    },

    #[error("expected `{expected}`, found `{actual}`")]
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
