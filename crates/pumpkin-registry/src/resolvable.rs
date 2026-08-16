use std::{
    any::type_name,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use pumpkin_util::identifier::Identifier;

use crate::{Registry, error::DataKeyGetError, value::DataKeyRef};

/// A typed registry entry that can be stored before the target registry is
/// available.
///
/// The resolvable intentionally only stores the entry identifier. The caller
/// selects the registry at resolution time, which keeps it usable for reloadable
/// registries and registry identifiers that contain slashes.
pub struct RegistryResolvable<T: Send + Sync + 'static> {
    identifier: Identifier,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> Clone for RegistryResolvable<T> {
    fn clone(&self) -> Self {
        Self::new(self.identifier.clone())
    }
}

impl<T: Send + Sync + 'static> Debug for RegistryResolvable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RegistryResolvable")
            .field(&self.identifier)
            .finish()
    }
}

impl<T: Send + Sync + 'static> PartialEq for RegistryResolvable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<T: Send + Sync + 'static> Eq for RegistryResolvable<T> {}

impl<T: Send + Sync + 'static> Hash for RegistryResolvable<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl<T: Send + Sync + 'static> RegistryResolvable<T> {
    #[must_use]
    pub const fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            marker: PhantomData,
        }
    }

    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn resolve<'a>(
        &self,
        registry: &'a dyn Registry,
    ) -> Result<DataKeyRef<'a, T>, DataKeyGetError> {
        let id = registry.get_id(&self.identifier).ok_or_else(|| {
            DataKeyGetError::MissingIdentifier {
                identifier: self.identifier.clone(),
            }
        })?;

        let value = registry
            .by_id_erased(id)
            .ok_or(DataKeyGetError::MissingValue { id })?;
        let typed = value
            .downcast_ref::<T>()
            .ok_or(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: registry.item_type_name(),
            })?;

        Ok(DataKeyRef {
            value: std::ptr::from_ref(typed),
            _guards: vec![value],
            _registry: None,
            marker: PhantomData,
        })
    }

    pub async fn resolve_async<'a>(
        &self,
        registry: &'a dyn Registry,
    ) -> Result<DataKeyRef<'a, T>, DataKeyGetError> {
        let id = registry
            .get_id_async(&self.identifier)
            .await
            .ok_or_else(|| DataKeyGetError::MissingIdentifier {
                identifier: self.identifier.clone(),
            })?;

        let value = registry
            .by_id_erased_async(id)
            .await
            .ok_or(DataKeyGetError::MissingValue { id })?;
        let typed = value
            .downcast_ref::<T>()
            .ok_or(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: registry.item_type_name(),
            })?;

        Ok(DataKeyRef {
            value: std::ptr::from_ref(typed),
            _guards: vec![value],
            _registry: None,
            marker: PhantomData,
        })
    }
}

/// A set of typed registry entries or a registry tag.
///
/// Tags remain unresolved until the registry layer gains tag support. This
/// type still preserves the distinction so datapack codecs can round-trip
/// `#namespace:tag` without treating it as an entry identifier.
pub enum RegistryResolvableSet<T: Send + Sync + 'static> {
    Single(RegistryResolvable<T>),
    Tag(Identifier),
    List(Box<[RegistryResolvable<T>]>),
}

impl<T: Send + Sync + 'static> Clone for RegistryResolvableSet<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Single(value) => Self::Single(value.clone()),
            Self::Tag(identifier) => Self::Tag(identifier.clone()),
            Self::List(values) => Self::List(values.to_vec().into_boxed_slice()),
        }
    }
}

impl<T: Send + Sync + 'static> Debug for RegistryResolvableSet<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(value) => f.debug_tuple("Single").field(value).finish(),
            Self::Tag(identifier) => f.debug_tuple("Tag").field(identifier).finish(),
            Self::List(values) => f.debug_tuple("List").field(values).finish(),
        }
    }
}

impl<T: Send + Sync + 'static> PartialEq for RegistryResolvableSet<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Single(left), Self::Single(right)) => left == right,
            (Self::Tag(left), Self::Tag(right)) => left == right,
            (Self::List(left), Self::List(right)) => left == right,
            _ => false,
        }
    }
}

impl<T: Send + Sync + 'static> Eq for RegistryResolvableSet<T> {}

impl<T: Send + Sync + 'static> Hash for RegistryResolvableSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Single(value) => value.hash(state),
            Self::Tag(identifier) => identifier.hash(state),
            Self::List(values) => values.hash(state),
        }
    }
}

impl<T: Send + Sync + 'static> RegistryResolvableSet<T> {
    #[must_use]
    pub const fn single(value: RegistryResolvable<T>) -> Self {
        Self::Single(value)
    }

    #[must_use]
    pub const fn tag(identifier: Identifier) -> Self {
        Self::Tag(identifier)
    }

    #[must_use]
    pub fn list<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = RegistryResolvable<T>>,
    {
        Self::List(iterator.into_iter().collect::<Vec<_>>().into_boxed_slice())
    }

    #[must_use]
    pub const fn as_single(&self) -> Option<&RegistryResolvable<T>> {
        match self {
            Self::Single(value) => Some(value),
            Self::Tag(_) | Self::List(_) => None,
        }
    }

    #[must_use]
    pub const fn as_tag(&self) -> Option<&Identifier> {
        match self {
            Self::Tag(identifier) => Some(identifier),
            Self::Single(_) | Self::List(_) => None,
        }
    }

    #[must_use]
    pub fn as_list(&self) -> Option<&[RegistryResolvable<T>]> {
        match self {
            Self::List(values) => Some(values),
            Self::Single(_) | Self::Tag(_) => None,
        }
    }
}
