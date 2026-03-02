use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const VANILLA_NAMESPACE: &str = "minecraft";
pub const PUMPKIN_NAMESPACE: &str = "pumpkin";

/// The type a part of an identifier is stored as.
pub type IdentifierPart = Cow<'static, str>;

/// An immutable structure that identifies a particular resource.
///
/// Identifiers are expressed in the form `<namespace>:<path>`.
///
/// The namespace may only contain:
///     - digits `[0-9]`
///     - lowercase letters `[a-z]`
///     - periods `.`
///     - underscores `_`
///     - hyphens `-`
///
/// The path allows all the characters that the namespace does, but
/// with the addition of forward slashes `/` (path separator).
///
/// If an identifier is specified without a colon and namespace,
/// i.e. just `<path>`, then the namespace is assumed
/// to be `minecraft`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Identifier {
    namespace: IdentifierPart,
    path: IdentifierPart,
}

/// Represents an error arising from
/// trying to create/parse an identifier.
#[derive(Clone, Debug, Error)]
pub enum IdentifierError {
    #[error("Invalid character in namespace of identifier: {0}")]
    InvalidNamespace(Identifier),

    #[error("Invalid character in path of identifier: {0}")]
    InvalidPath(Identifier),
}

/// Represents a result of an attempt to create an [`Identifier`].
pub type IdentifierCreationResult = Result<Identifier, IdentifierError>;

impl Identifier {
    /// Tries to create a new [`Identifier`] from both a specified namespace and path.
    #[must_use]
    pub fn new(
        namespace: impl Into<IdentifierPart>,
        path: impl Into<IdentifierPart>,
    ) -> IdentifierCreationResult {
        let namespace = namespace.into();
        let path = path.into();

        let identifier = Self { namespace, path };

        Self::validate_identifier(identifier)
    }

    /// Creates a new [`Identifier`] from both a specified namespace and path
    /// at compile-time.
    ///
    /// # Panics
    ///
    /// Panics if either the provided namespace or provided path is invalid.
    #[must_use]
    pub const fn from_static(namespace: &'static str, path: &'static str) -> Self {
        // We cannot directly use `IdentifierCreationError`
        // here as we're in a const context. We cannot use
        // actual formatting either.
        assert!(
            Self::is_valid_namespace(namespace),
            "Invalid character in namespace of identifier"
        );
        assert!(
            Self::is_valid_path(path),
            "Invalid character in path of identifier"
        );

        Self {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
        }
    }

    /// Attempts to parse an identifier from a given string.
    #[must_use]
    pub fn parse(identifier: &str) -> IdentifierCreationResult {
        let colon_i = identifier.bytes().position(|b| b == b':');

        if let Some(colon_i) = colon_i {
            // Colon exists.
            let path = identifier[colon_i + 1..].to_string();

            if colon_i == 0 {
                Self::new(VANILLA_NAMESPACE, path)
            } else {
                let namespace = identifier[0..colon_i].to_string();
                Self::new(namespace, path)
            }
        } else {
            Self::new(VANILLA_NAMESPACE, identifier.to_string())
        }
    }

    // Attempts to parse an identifier from a given string at compile-time.
    #[must_use]
    pub const fn parse_static(identifier: &'static str) -> Identifier {
        let bytes = identifier.as_bytes();
        let mut colon_i = 0;

        while colon_i < bytes.len() {
            if bytes[colon_i] == b':' {
                break;
            }
            colon_i += 1;
        }

        if colon_i < bytes.len() {
            // Colon exists.

            let path = unsafe {
                // SAFETY: The given start and end is valid.
                // `colon_i` is at a ':', which is a one-byte character.
                Self::slice_bytes_to_str_unchecked(bytes, colon_i + 1, bytes.len())
            };

            if colon_i == 0 {
                Self::from_static(VANILLA_NAMESPACE, path)
            } else {
                let namespace = unsafe {
                    // SAFETY: The given start and end is valid.
                    // `colon_i` is at a ':', which is a one-byte character.
                    Self::slice_bytes_to_str_unchecked(bytes, 0, colon_i)
                };
                Self::from_static(namespace, path)
            }
        } else {
            Self::from_static(VANILLA_NAMESPACE, identifier)
        }
    }

    /// Unsafe method to slice bytes into a `&str` at valid positions.
    /// We do this as `std::ops::Index` is not yet stable as a const trait.
    ///
    /// `start` and `end` are expressed in bytes.
    ///
    /// The start and end must be valid for the slice and must not slice
    /// in a multi-byte character, as we need valid UTF-8.
    const unsafe fn slice_bytes_to_str_unchecked(bytes: &[u8], start: usize, end: usize) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                bytes.as_ptr().add(start),
                end - start,
            ))
        }
    }

    /// Tries to create a new [`Identifier`] that has a namespace of `minecraft`.
    pub fn vanilla(path: impl Into<IdentifierPart>) -> IdentifierCreationResult {
        Self::new(VANILLA_NAMESPACE, path)
    }

    /// Tries to create a new [`Identifier`] that has a namespace of `minecraft` at compile-time.
    ///
    /// # Panics
    ///
    /// Panics if the provided path is invalid.
    #[must_use]
    pub const fn vanilla_static(path: &'static str) -> Self {
        Self::from_static(VANILLA_NAMESPACE, path)
    }

    /// Creates a new [`Identifier`] that has a namespace of `pumpkin`.
    pub fn pumpkin(path: impl Into<IdentifierPart>) -> IdentifierCreationResult {
        Self::new(PUMPKIN_NAMESPACE, path)
    }

    /// Creates a new [`Identifier`] that has a namespace of `pumpkin` at compile-time.
    ///
    /// # Panics
    ///
    /// Panics if the provided path is invalid.
    #[must_use]
    pub const fn pumpkin_static(path: &'static str) -> Self {
        Self::from_static(PUMPKIN_NAMESPACE, path)
    }

    /// Consumes this identifier to create a new one with the specified path.
    pub fn with_path(self, path: impl Into<IdentifierPart>) -> IdentifierCreationResult {
        Self::validate_identifier_only_path(Self {
            namespace: self.namespace,
            path: path.into(),
        })
    }

    /// Consumes this identifier to add a prefix to the current path.
    pub fn prefix_path(self, prefix: &str) -> IdentifierCreationResult {
        Self::validate_identifier_only_path(Self {
            namespace: self.namespace,
            path: Cow::Owned(format!("{prefix}{}", self.path)),
        })
    }

    /// Consumes this identifier to add a suffix to the current path.
    pub fn suffix_path(self, suffix: &str) -> IdentifierCreationResult {
        Self::validate_identifier_only_path(Self {
            namespace: self.namespace,
            path: Cow::Owned(format!("{}{suffix}", self.path)),
        })
    }

    /// Consumes this identifier to create a new one by mapping its path through the given function.
    pub fn map_path<F>(self, f: F) -> IdentifierCreationResult
    where
        F: FnOnce(&str) -> String,
    {
        Self::validate_identifier_only_path(Self {
            namespace: self.namespace,
            path: Cow::Owned(f(&self.path)),
        })
    }

    /// Gets the namespace of this [`Identifier`].
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Gets the path of this [`Identifier`].
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    fn validate_identifier(identifier: Identifier) -> IdentifierCreationResult {
        if !Self::is_valid_namespace(&identifier.namespace) {
            return Err(IdentifierError::InvalidNamespace(identifier));
        }
        if !Self::is_valid_path(&identifier.path) {
            return Err(IdentifierError::InvalidPath(identifier));
        }
        Ok(identifier)
    }

    #[must_use]
    fn validate_identifier_only_path(identifier: Identifier) -> IdentifierCreationResult {
        if !Self::is_valid_path(&identifier.path) {
            return Err(IdentifierError::InvalidPath(identifier));
        }
        Ok(identifier)
    }

    #[must_use]
    /// Returns whether the given namespace would be valid if
    /// used in an identifier.
    pub const fn is_valid_namespace(namespace: &str) -> bool {
        // We have to use a manual loop so that the function
        // can be marked as a `const` function.
        let bytes = namespace.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if !matches!(bytes[i], b'0'..=b'9' | b'a'..=b'z' | b'-' | b'_' | b'.') {
                return false;
            }
            i += 1;
        }
        true
    }

    #[must_use]
    /// Returns whether the given path would be valid if
    /// used in an identifier.
    pub const fn is_valid_path(path: &str) -> bool {
        // We have to use a manual loop so that the function
        // can be marked as a `const` function.
        let bytes = path.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if !matches!(bytes[i], b'0'..=b'9' | b'a'..=b'z' | b'-' | b'_' | b'.' | b'/') {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let identifier_string = String::deserialize(deserializer)?;
        Ok(Identifier::parse(&identifier_string)
            .map_err(|error| serde::de::Error::custom(error.to_string()))?)
    }
}
