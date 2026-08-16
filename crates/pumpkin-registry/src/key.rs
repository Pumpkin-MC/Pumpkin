use crate::{Registry, error::DataKeyGetError, value::DataKeyRef};
use pumpkin_util::identifier::Identifier;
use std::{
    borrow::Cow,
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

pub struct DataKey<T: Send + Sync + 'static> {
    path: Cow<'static, str>,
    ids: OnceLock<Box<[usize]>>,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> DataKey<T> {
    #[must_use]
    pub const fn new(path: &'static str) -> Self {
        // Ideally const-validate the path here.
        Self {
            path: Cow::Borrowed(path),
            ids: OnceLock::new(),
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn owned(path: String) -> Self {
        Self {
            path: Cow::Owned(path),
            ids: OnceLock::new(),
            marker: PhantomData,
        }
    }

    pub fn get<'a>(&self, root: &'a dyn Registry) -> Result<DataKeyRef<'a, T>, DataKeyGetError> {
        let ids = if let Some(ids) = self.ids.get() {
            ids
        } else {
            let ids = resolve_key_path(root, self.path.as_ref())?;
            // Another caller may have resolved it concurrently.
            let _ = self.ids.set(ids);
            // Either ours or the concurrently initialized value.
            #[allow(clippy::expect_used)]
            self.ids.get().expect("DataKey ids were initialized")
        };

        get_from_key(root, ids)
    }
}

enum RegistryCursor<'a> {
    Borrowed(&'a dyn Registry),
    Owned(Arc<dyn Registry>),
}

impl RegistryCursor<'_> {
    fn as_ref(&self) -> &dyn Registry {
        match self {
            Self::Borrowed(registry) => *registry,
            Self::Owned(registry) => registry.as_ref(),
        }
    }
}

struct PathComponents<'a> {
    remaining: &'a str,
}

impl<'a> PathComponents<'a> {
    const fn new(path: &'a str) -> Self {
        Self { remaining: path }
    }
}

impl Iterator for PathComponents<'_> {
    type Item = Result<Identifier, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let (current, rest) = match self.remaining.find('/') {
            Some(index) => (&self.remaining[..index], &self.remaining[index + 1..]),
            None => (self.remaining, ""),
        };

        self.remaining = rest;

        Some(Identifier::parse(current).map_err(|_| ()))
    }
}

fn resolve_key_path(root: &dyn Registry, path: &str) -> Result<Box<[usize]>, DataKeyGetError> {
    let mut identifiers = PathComponents::new(path).peekable();
    let mut ids = Vec::new();
    let mut current = RegistryCursor::Borrowed(root);

    while let Some(identifier) = identifiers.next() {
        let identifier = identifier.map_err(|()| DataKeyGetError::InvalidKey)?;
        let registry = current.as_ref();

        let id = registry
            .get_id(&identifier)
            .ok_or(DataKeyGetError::MissingIdentifier { identifier })?;

        ids.push(id);

        // Last component is the actual value.
        if identifiers.peek().is_none() {
            return Ok(ids.into_boxed_slice());
        }

        let value = registry
            .by_id_erased(id)
            .ok_or(DataKeyGetError::MissingRegistry { id })?;

        let nested = value
            .downcast_ref::<Arc<dyn Registry>>()
            .ok_or(DataKeyGetError::MissingRegistry { id })?;

        let nested = Arc::clone(nested);

        drop(value);

        current = RegistryCursor::Owned(nested);
    }

    Err(DataKeyGetError::InvalidKey)
}

fn get_from_key<'a, T>(
    root: &'a dyn Registry,
    ids: &[usize],
) -> Result<DataKeyRef<'a, T>, DataKeyGetError>
where
    T: Send + Sync + 'static,
{
    let Some((&value_id, registry_path)) = ids.split_last() else {
        return Err(DataKeyGetError::InvalidKey);
    };

    let mut current = RegistryCursor::Borrowed(root);

    for &registry_id in registry_path {
        let value = current
            .as_ref()
            .by_id_erased(registry_id)
            .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

        let nested = value
            .downcast_ref::<Arc<dyn Registry>>()
            .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

        let nested = Arc::clone(nested);

        drop(value);

        current = RegistryCursor::Owned(nested);
    }

    match current {
        RegistryCursor::Borrowed(registry) => {
            let value = registry
                .by_id_erased(value_id)
                .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

            let typed = value
                .downcast_ref::<T>()
                .ok_or(DataKeyGetError::TypeMismatch {
                    expected: std::any::type_name::<T>(),
                    actual: registry.item_type_name(),
                })?;

            Ok(DataKeyRef {
                value: std::ptr::from_ref(typed),
                _guards: vec![value],
                _registry: None,
                marker: PhantomData,
            })
        }

        RegistryCursor::Owned(registry) => {
            let value = registry
                .by_id_erased(value_id)
                .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

            let typed = value
                .downcast_ref::<T>()
                .ok_or(DataKeyGetError::TypeMismatch {
                    expected: std::any::type_name::<T>(),
                    actual: registry.item_type_name(),
                })?;

            let value_ptr = std::ptr::from_ref(typed);

            // SAFETY:
            //
            // `value` borrows from `registry`. The returned `DataKeyRef`
            // stores the owning Arc in `_registry`, keeping that registry
            // alive for at least as long as the erased reference.
            //
            // `_guards` must also be dropped before `_registry`.
            let value = unsafe {
                std::mem::transmute::<
                    crate::value::ErasedRegistryRef<'_>,
                    crate::value::ErasedRegistryRef<'a>,
                >(value)
            };

            Ok(DataKeyRef {
                value: value_ptr,
                _guards: vec![value],
                _registry: Some(registry),
                marker: PhantomData,
            })
        }
    }
}
