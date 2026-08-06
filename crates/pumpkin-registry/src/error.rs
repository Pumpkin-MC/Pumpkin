use pumpkin_util::identifier::Identifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryInsertError {
    #[error("registry entry `{0}` is already registered")]
    AlreadyRegistered(Identifier),
    #[error("registry is immutable")]
    Immutable,
}

#[derive(Debug, Error)]
pub enum RegistryInitError {
    #[error(
        "the amount of values ({values}) doesn't match the amount of identifiers ({identifiers})"
    )]
    MappingMismatch { values: usize, identifiers: usize },

    #[error("registry entry `{0}` is already registered")]
    AlreadyRegistered(Identifier),
}

#[derive(Debug, Error)]
pub enum RegistryGetError {
    #[error("registry path cannot be empty")]
    EmptyPath,

    #[error("registry entry `{0}` was not found")]
    NotFound(Identifier),

    #[error("registry entry `{0}` is not a nested registry")]
    ExpectedRegistry(Identifier),

    #[error("registry `{identifier}` has the wrong entry type; expected `{expected}`")]
    TypeMismatch {
        identifier: Identifier,
        expected: &'static str,
    },
}

#[derive(Debug, thiserror::Error)]
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

#[derive(Debug, thiserror::Error)]
pub enum DataKeyGetError {
    #[error("the registry used to build this key was dropped")]
    RegistryDropped,

    #[error("this key belongs to a different registry tree")]
    WrongRegistry,

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
