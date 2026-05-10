use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const VANILLA_NAMESPACE: &str = "minecraft";
pub const PUMPKIN_NAMESPACE: &str = "pumpkin";

/// An immutable structure that identifies a particular resource,
/// which is possibly heap-allocated.
/// They are expressed in the form `<namespace>:<path>`.
///
/// The namespace may only contain:
/// - digits `[0-9]`
/// - lowercase letters `[a-z]`
/// - periods `.`
/// - underscores `_`
/// - hyphens `-`
///
/// The path allows all the characters that the namespace does, but
/// with the addition of forward slashes `/` (path separator).
///
/// If an identifier is specified without a colon and namespace,
/// i.e. just `<path>`, then the namespace is assumed
/// to be `minecraft`.
///
/// # Implementation Note
///
/// Namespace and path are stored separately, internally
/// as a `Cow<'static, str>`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Identifier {
    namespace: Cow<'static, str>,
    path: Cow<'static, str>,
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
    pub fn new(
        namespace: impl Into<Cow<'static, str>>,
        path: impl Into<Cow<'static, str>>,
    ) -> IdentifierCreationResult {
        let namespace = namespace.into();
        let path = path.into();

        let identifier = Self { namespace, path };

        Self::validate_identifier(identifier)
    }

    /// Creates a new [`Identifier`] from both a specified namespace and path
    /// at compile-time.
    ///
    /// The identifier returned is guaranteed **NOT to be heap allocated**.
    ///
    /// # Panics
    ///
    /// Panics if either the provided namespace or provided path is invalid.
    #[must_use]
    pub const fn from_static(namespace: &'static str, path: &'static str) -> Self {
        assert!(
            Self::is_valid_namespace(namespace),
            "Invalid namespace provided"
        );
        assert!(Self::is_valid_path(path), "Invalid path provided");

        Self {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
        }
    }

    /// Attempts to parse an identifier from a given string.
    ///
    /// The identifier returned is heap allocated.
    pub fn parse(identifier: &str) -> IdentifierCreationResult {
        identifier.bytes().position(|b| b == b':').map_or_else(
            || Self::new(VANILLA_NAMESPACE, identifier.to_string()),
            |colon_i| {
                // Colon exists.
                let path = identifier[colon_i + 1..].to_string();

                if colon_i == 0 {
                    Self::new(VANILLA_NAMESPACE, path)
                } else {
                    let namespace = identifier[0..colon_i].to_string();
                    Self::new(namespace, path)
                }
            },
        )
    }

    /// Attempts to parse an identifier from a given string at compile-time.
    ///
    /// The identifier returned is guaranteed **NOT to be heap allocated**.
    #[must_use]
    pub const fn parse_static(identifier: &'static str) -> Self {
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
            // We are forced to use unsafe code in a const
            // as Index trait is not const.

            let path = unsafe {
                // SAFETY: The given start and end is valid.
                // `colon_i` is at a ':', which is a one-byte ASCII character (0x3A)
                // This means that colon_i + 1 is a valid boundary.
                // Rust guarantees that `identifier` is a valid UTF-8 string.
                Self::slice_bytes_to_str_unchecked(bytes, colon_i + 1, bytes.len())
            };

            if colon_i == 0 {
                Self::from_static(VANILLA_NAMESPACE, path)
            } else {
                let namespace = unsafe {
                    // SAFETY: The given start and end is valid.
                    // `colon_i` is at a ':', which is a one-byte ASCII character (0x3A)
                    // This means that colon_i is a valid boundary.
                    // Rust guarantees that `identifier` is a valid UTF-8 string.
                    Self::slice_bytes_to_str_unchecked(bytes, 0, colon_i)
                };
                Self::from_static(namespace, path)
            }
        } else {
            Self::from_static(VANILLA_NAMESPACE, identifier)
        }
    }

    /// Unsafe function to slice bytes into a `&str` at valid positions.
    /// We do this as `std::ops::Index` is not yet stable as a const trait.
    ///
    /// `start` and `end` are expressed in bytes, acting as indices for `bytes`.
    ///
    /// # Safety
    /// Undefined behavior occurs if any one of the following conditions are violated:
    /// - `start <= end <= bytes.len()`
    /// - `start` and `end` both lie on UTF-8 character boundaries.
    /// - the subslice `bytes[start..end]` is valid UTF-8.
    #[must_use]
    const unsafe fn slice_bytes_to_str_unchecked(bytes: &[u8], start: usize, end: usize) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                bytes.as_ptr().add(start),
                end - start,
            ))
        }
    }

    /// Tries to create a new [`Identifier`] that has a namespace of `minecraft`.
    pub fn vanilla(path: impl Into<Cow<'static, str>>) -> IdentifierCreationResult {
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
    pub fn pumpkin(path: impl Into<Cow<'static, str>>) -> IdentifierCreationResult {
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

    /// Consumes this identifier to replace its current path with the specified path.
    pub fn with_path(self, path: impl Into<Cow<'static, str>>) -> IdentifierCreationResult {
        Self::validate_identifier_path(Self {
            namespace: self.namespace,
            path: path.into(),
        })
    }

    /// Consumes this identifier to add a prefix to the current path.
    pub fn prefix_path(self, prefix: &str) -> IdentifierCreationResult {
        Self::validate_identifier_path(Self {
            namespace: self.namespace,
            path: format!("{prefix}{}", self.path).into(),
        })
    }

    /// Consumes this identifier to add a suffix to the current path.
    pub fn suffix_path(self, suffix: &str) -> IdentifierCreationResult {
        Self::validate_identifier_path(Self {
            namespace: self.namespace,
            path: format!("{}{suffix}", self.path).into(),
        })
    }

    /// Consumes this identifier to create a new one by mapping its path through the given function.
    pub fn map_path<F>(self, f: F) -> IdentifierCreationResult
    where
        F: FnOnce(&str) -> String,
    {
        Self::validate_identifier_path(Self {
            namespace: self.namespace,
            path: f(&self.path).into(),
        })
    }

    /// Gets the namespace of this [`Identifier`].
    #[must_use]
    #[inline]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Gets the path of this [`Identifier`].
    #[must_use]
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    fn validate_identifier(identifier: Self) -> IdentifierCreationResult {
        if !Self::is_valid_namespace(&identifier.namespace) {
            return Err(IdentifierError::InvalidNamespace(identifier));
        }
        if !Self::is_valid_path(&identifier.path) {
            return Err(IdentifierError::InvalidPath(identifier));
        }
        Ok(identifier)
    }

    fn validate_identifier_path(identifier: Self) -> IdentifierCreationResult {
        if !Self::is_valid_path(&identifier.path) {
            return Err(IdentifierError::InvalidPath(identifier));
        }
        Ok(identifier)
    }

    /// Returns whether the given namespace would be valid if
    /// used in an identifier.
    #[must_use]
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

    /// Returns whether the given path would be valid if
    /// used in an identifier.
    #[must_use]
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

    /// Returns whether the given character could be valid in an identifier.
    #[must_use]
    pub const fn is_valid_char(c: char) -> bool {
        matches!(c, '0'..='9' | 'a'..='z' | '-' | '_' | '.' | '/' | ':')
    }

    /// Gets a tuple of references to the internal strings of the
    /// namespace and path.
    #[must_use]
    pub fn view(&self) -> (&str, &str) {
        (&self.namespace, &self.path)
    }

    /// Returns whether this identifier is a `minecraft:` prefixed one.
    #[must_use]
    pub fn is_vanilla(&self) -> bool {
        self.namespace() == VANILLA_NAMESPACE
    }

    /// Returns whether this identifier is a `pumpkin:` prefixed one.
    #[must_use]
    pub fn is_pumpkin(&self) -> bool {
        self.namespace() == PUMPKIN_NAMESPACE
    }

    /// If this identifier is a `minecraft:` prefixed one, it returns
    /// a [`Some`] containing the path of this identifier. Otherwise,
    /// a [`None`] is returned.
    #[must_use]
    pub fn is_vanilla_then(&self) -> Option<&str> {
        self.is_vanilla().then_some(&self.path)
    }

    /// If this identifier is a `pumpkin:` prefixed one, it returns
    /// a [`Some`] containing the path of this identifier. Otherwise,
    /// a [`None`] is returned.
    #[must_use]
    pub fn is_pumpkin_then(&self) -> Option<&str> {
        self.is_pumpkin().then_some(&self.path)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = IdentifierError;

    fn try_from(value: &str) -> IdentifierCreationResult {
        Self::parse(value)
    }
}

impl TryFrom<&String> for Identifier {
    type Error = IdentifierError;

    fn try_from(value: &String) -> IdentifierCreationResult {
        Self::parse(value)
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
        Self::parse(&identifier_string).map_err(|error| serde::de::Error::custom(error.to_string()))
    }
}

#[cfg(test)]
mod test {
    use crate::identifier::{Identifier, IdentifierError};

    #[test]
    fn new() -> Result<(), IdentifierError> {
        assert_eq!(Identifier::new("abc", "def")?.to_string(), "abc:def");
        assert_eq!(Identifier::from_static("abc", "def").to_string(), "abc:def");

        Ok(())
    }

    #[test]
    fn parse() -> Result<(), IdentifierError> {
        assert_eq!(Identifier::parse("abc")?.to_string(), "minecraft:abc");
        assert_eq!(Identifier::parse("abc:def")?.to_string(), "abc:def");
        assert_eq!(Identifier::parse("abc:")?.to_string(), "abc:");
        assert_eq!(Identifier::parse(":def")?.to_string(), "minecraft:def");
        assert_eq!(Identifier::parse(":")?.to_string(), "minecraft:");

        assert_eq!(Identifier::parse_static("abc").to_string(), "minecraft:abc");
        assert_eq!(Identifier::parse_static("abc:def").to_string(), "abc:def");

        let _ = Identifier::parse("")?;
        let _ = Identifier::parse("abc:/4/5")?;
        let _ = Identifier::parse("a._b-c:/4_-/5.9")?;

        assert!(Identifier::parse("::").is_err());
        assert!(Identifier::parse("a:b:c").is_err());
        assert!(Identifier::parse("he/llo:bye").is_err());
        assert!(Identifier::parse("1234+567:89").is_err());

        Ok(())
    }
}
