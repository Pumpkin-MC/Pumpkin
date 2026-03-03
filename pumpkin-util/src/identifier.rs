use std::{cmp::Ordering, fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const VANILLA_NAMESPACE: &str = "minecraft";
pub const PUMPKIN_NAMESPACE: &str = "pumpkin";

/// The type a part of an identifier is stored as.
/// 
/// This is an implementation detail.
#[derive(Clone, Debug)]
pub enum IdentifierPart {
    Static(&'static str),
    Box(Box<str>)
}

impl Deref for IdentifierPart {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Static(string) => string,
            Self::Box(string) => string
        }
    }
}

impl PartialEq for IdentifierPart {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl Eq for IdentifierPart {}

impl PartialOrd for IdentifierPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for IdentifierPart {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}   

impl std::hash::Hash for IdentifierPart {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl std::fmt::Display for IdentifierPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl From<&'static str> for IdentifierPart {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<Box<str>> for IdentifierPart {
    fn from(value: Box<str>) -> Self {
        Self::Box(value)
    }
}

impl From<String> for IdentifierPart {
    fn from(value: String) -> Self {
        value.into_boxed_str().into()
    }
}

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
/// 
/// The static version of this struct is [`StaticIdentifier`], whose
/// methods are `const fn`s.
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
        StaticIdentifier::new(namespace, path).as_identifier()
    }

    /// Attempts to parse an identifier from a given string.
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

    // Attempts to parse an identifier from a given string at compile-time.
    #[must_use]
    pub const fn parse_static(identifier: &'static str) -> Self {
        StaticIdentifier::parse(identifier).as_identifier()
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
            path: format!("{prefix}{}", self.path).into(),
        })
    }

    /// Consumes this identifier to add a suffix to the current path.
    pub fn suffix_path(self, suffix: &str) -> IdentifierCreationResult {
        Self::validate_identifier_only_path(Self {
            namespace: self.namespace,
            path: format!("{}{suffix}", self.path).into(),
        })
    }

    /// Consumes this identifier to create a new one by mapping its path through the given function.
    pub fn map_path<F>(self, f: F) -> IdentifierCreationResult
    where
        F: FnOnce(&str) -> String,
    {
        Self::validate_identifier_only_path(Self {
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

    fn validate_identifier_only_path(identifier: Self) -> IdentifierCreationResult {
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

/// Allows a type to convert an object of its own to and fro its identifier.
pub trait Identified: Sized {
    /// Uses a registry to find an object from its own type from its [`Identifier`].
    fn from_identifier(identifier: &Identifier) -> Option<Self>;

    /// Queries the [`Identifier`] of this object.
    fn to_identifier(&self) -> Identifier;
}

pub trait StaticallyIdentified: Sized {
    /// Uses a registry to find an object from its own type from its [`Identifier`].
    fn from_identifier(identifier: &Identifier) -> Option<Self>;

    /// Queries the [`Identifier`] of this object.
    fn to_identifier(&self) -> Identifier;
}

/// A version of [`Identifier`] only composed of `&static str`s.
/// 
/// All the useful methods of this struct are `const fn`s.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct StaticIdentifier {
    namespace: &'static str,
    path: &'static str,
}

impl StaticIdentifier {
    /// Creates a new [`StaticIdentifier`] from
    /// a namespace and path.
    /// 
    /// # Panics
    /// 
    /// Panics if either the namespace or the path
    /// provider is invalid.
    #[must_use]
    #[inline]
    pub const fn new(
        namespace: &'static str,
        path: &'static str
    ) -> Self {
        assert!(
            Identifier::is_valid_namespace(namespace),
            "Invalid namespace provided"
        );
        assert!(
            Identifier::is_valid_path(path),
            "Invalid path provided"
        );

        Self {
            namespace,
            path
        }
    }

    #[must_use]
    pub const fn parse(identifier: &'static str) -> Self {
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
                // `colon_i` is at a ':', which is a one-byte character.
                Self::slice_bytes_to_str_unchecked(bytes, colon_i + 1, bytes.len())
            };

            if colon_i == 0 {
                Self::new(VANILLA_NAMESPACE, path)
            } else {
                let namespace = unsafe {
                    // SAFETY: The given start and end is valid.
                    // `colon_i` is at a ':', which is a one-byte character.
                    Self::slice_bytes_to_str_unchecked(bytes, 0, colon_i)
                };
                Self::new(namespace, path)
            }
        } else {
            Self::new(VANILLA_NAMESPACE, identifier)
        }
    }

    /// Converts this into a normal [`Identifier`].
    #[must_use]
    #[inline]
    pub const fn as_identifier(&self) -> Identifier {
        Identifier {
            namespace: IdentifierPart::Static(self.namespace),
            path: IdentifierPart::Static(self.path)
        }
    }

    /// Creates a new [`StaticIdentifier`] that has a namespace of `minecraft`.
    #[must_use]
    #[inline]
    pub const fn vanilla(path: &'static str) -> Self {
        Self::new(VANILLA_NAMESPACE, path)
    }

    /// Creates a new [`StaticIdentifier`] that has a namespace of `pumpkin`.
    #[must_use]
    #[inline]
    pub const fn pumpkin(path: &'static str) -> Self {
        Self::new(PUMPKIN_NAMESPACE, path)
    }

    /// Returns the namespace of this identifier.
    #[must_use]
    #[inline]
    pub const fn namespace(&self) -> &'static str {
        self.namespace
    }

    /// Returns the path of this identifier.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> &'static str {
        self.path
    }

    /// Returns a view to both the namespace and path of this identifier.
    #[must_use]
    #[inline]
    pub const fn view(&self) -> (&'static str, &'static str) {
        (self.namespace, self.path)
    }

    /// Returns whether this identifier is a `minecraft:` prefixed one.
    #[must_use]
    #[inline]
    pub const fn is_vanilla(&self) -> bool {
        Self::check_eq_const(self.namespace, VANILLA_NAMESPACE)
    }

    /// Returns whether this identifier is a `pumpkin:` prefixed one.
    #[must_use]
    #[inline]
    pub const fn is_pumpkin(&self) -> bool {
        Self::check_eq_const(self.namespace, PUMPKIN_NAMESPACE)
    }

    /// If this identifier is a `minecraft:` prefixed one, it returns
    /// a [`Some`] containing the path of this identifier. Otherwise,
    /// a [`None`] is returned.
    #[must_use]
    #[inline]
    pub const fn is_vanilla_then(&self) -> Option<&str> {
        if self.is_vanilla() {
            Some(self.path)
        } else {
            None
        }
    }

    /// If this identifier is a `pumpkin:` prefixed one, it returns
    /// a [`Some`] containing the path of this identifier. Otherwise,
    /// a [`None`] is returned.
    #[must_use]
    #[inline]
    pub const fn is_pumpkin_then(&self) -> Option<&str> {
        if self.is_pumpkin() {
            Some(self.path)
        } else {
            None
        }
    }

    /// Unsafe method to slice bytes into a `&str` at valid positions.
    /// We do this as `std::ops::Index` is not yet stable as a const trait.
    ///
    /// `start` and `end` are expressed in bytes.
    ///
    /// The start and end must be valid for the slice and must not slice
    /// in a multi-byte character, as we need valid UTF-8.
    #[must_use]
    const unsafe fn slice_bytes_to_str_unchecked(bytes: &[u8], start: usize, end: usize) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                bytes.as_ptr().add(start),
                end - start,
            ))
        }
    }

    /// Const method to check equality of two strings.
    /// `PartialEq` is not const unfortunately :(
    #[must_use]
    const fn check_eq_const(a: &str, b: &str) -> bool {
        let a = a.as_bytes();
        let b = b.as_bytes();

        if a.len() != b.len() {
            return false;
        }

        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }

        true
    }
}

impl From<StaticIdentifier> for Identifier {
    fn from(value: StaticIdentifier) -> Self {
        value.as_identifier()
    }
}

impl PartialEq<StaticIdentifier> for Identifier {
    fn eq(&self, other: &StaticIdentifier) -> bool {
        &*self.namespace == other.namespace && &*self.path == other.path
    }
}

impl PartialEq<Identifier> for StaticIdentifier {
    fn eq(&self, other: &Identifier) -> bool {
        self.namespace == &*other.namespace && self.path == &*other.path
    }
}