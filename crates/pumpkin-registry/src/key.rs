use crate::{
    BoxFuture, Registry,
    error::{DataKeyBuildError, DataKeyGetError},
    value::DataKeyRef,
};
use pumpkin_util::identifier::Identifier;
use std::{any::type_name, marker::PhantomData, ptr, sync::Arc};

pub trait DataKey<T>
where
    T: Send + Sync + 'static,
{
    /// Returns the numeric path, including the final value ID.
    fn ids(&self) -> &[usize];

    /// Returns the root registry this key belongs to.
    fn root_registry(&self) -> &dyn Registry;

    fn get(&self) -> BoxFuture<'_, Result<DataKeyRef<'_, T>, DataKeyGetError>> {
        get_from_key(self.root_registry(), self.ids())
    }

    /// Blocking lookup for synchronous callers.
    ///
    /// This parks the calling thread while a mutable registry is contended.
    /// Do not call it from a Tokio worker; async callers should use [`Self::get`].
    fn get_blocking(&self) -> Result<DataKeyRef<'_, T>, DataKeyGetError> {
        get_from_key_blocking(self.root_registry(), self.ids())
    }
}

fn get_from_key<'a, T>(
    root: &'a dyn Registry,
    keys: &'a [usize],
) -> BoxFuture<'a, Result<DataKeyRef<'a, T>, DataKeyGetError>>
where
    T: Send + Sync + 'static,
{
    Box::pin(async move {
        let Some((&value_id, registry_path)) = keys.split_last() else {
            return Err(DataKeyGetError::InvalidKey);
        };

        let mut current = RegistryCursor::Borrowed(root);

        for &registry_id in registry_path {
            let value = current
                .as_ref()
                .by_id_erased_async(registry_id)
                .await
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

            let nested = value
                .downcast_ref::<Arc<dyn Registry>>()
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

            let nested = Arc::clone(nested);

            // `value` may borrow from `current`, so release it before
            // replacing `current`.
            drop(value);

            current = RegistryCursor::Owned(nested);
        }

        match current {
            RegistryCursor::Borrowed(registry) => {
                let value = registry
                    .by_id_erased_async(value_id)
                    .await
                    .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

                let typed = value
                    .downcast_ref::<T>()
                    .ok_or(DataKeyGetError::TypeMismatch {
                        expected: type_name::<T>(),
                        actual: registry.item_type_name(),
                    })?;

                Ok(DataKeyRef {
                    value: ptr::from_ref(typed),
                    _guards: vec![value],
                    _registry: None,
                    marker: PhantomData,
                })
            }
            RegistryCursor::Owned(registry) => {
                let value = registry
                    .by_id_erased_async(value_id)
                    .await
                    .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

                let typed = value
                    .downcast_ref::<T>()
                    .ok_or(DataKeyGetError::TypeMismatch {
                        expected: type_name::<T>(),
                        actual: registry.item_type_name(),
                    })?;
                let value_ptr = ptr::from_ref(typed);

                // SAFETY: `value` only borrows from `registry`. The returned
                // DataKeyRef owns that Arc in `_registry`, and `_guards` is
                // declared before `_registry`, so the borrow is dropped first.
                let value = unsafe {
                    std::mem::transmute::<
                        crate::value::ErasedRegistryRef<'_>,
                        crate::value::ErasedRegistryRef<'a>,
                    >(value)
                };

                Ok(DataKeyRef {
                    _guards: vec![value],
                    _registry: Some(registry),
                    value: value_ptr,
                    marker: PhantomData,
                })
            }
        }
    })
}

fn get_from_key_blocking<'a, T>(
    root: &'a dyn Registry,
    keys: &'a [usize],
) -> Result<DataKeyRef<'a, T>, DataKeyGetError>
where
    T: Send + Sync + 'static,
{
    let Some((&value_id, registry_path)) = keys.split_last() else {
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
                    expected: type_name::<T>(),
                    actual: registry.item_type_name(),
                })?;

            Ok(DataKeyRef {
                value: ptr::from_ref(typed),
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
                    expected: type_name::<T>(),
                    actual: registry.item_type_name(),
                })?;
            let value_ptr = ptr::from_ref(typed);

            // SAFETY: same invariant as in the async path above.
            let value = unsafe {
                std::mem::transmute::<
                    crate::value::ErasedRegistryRef<'_>,
                    crate::value::ErasedRegistryRef<'a>,
                >(value)
            };

            Ok(DataKeyRef {
                _guards: vec![value],
                _registry: Some(registry),
                value: value_ptr,
                marker: PhantomData,
            })
        }
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

pub struct ArcDataKey<T: Send + Sync + 'static> {
    keys: Box<[usize]>,
    root: Arc<dyn Registry>,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> Clone for ArcDataKey<T> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            root: Arc::clone(&self.root),
            marker: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> DataKey<T> for ArcDataKey<T> {
    fn ids(&self) -> &[usize] {
        &self.keys
    }

    fn root_registry(&self) -> &dyn Registry {
        self.root.as_ref()
    }
}

pub struct RefDataKey<'a, T: Send + Sync + 'static> {
    keys: Box<[usize]>,
    root: &'a dyn Registry,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> Clone for RefDataKey<'_, T> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            root: self.root,
            marker: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> DataKey<T> for RefDataKey<'_, T> {
    fn ids(&self) -> &[usize] {
        &self.keys
    }

    fn root_registry(&self) -> &dyn Registry {
        self.root
    }
}

pub struct DataKeyBuilder<T: Send + Sync + 'static> {
    keys: Vec<Identifier>,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> DataKeyBuilder<T> {
    #[must_use]
    pub fn new(identifier: Identifier) -> Self {
        Self {
            keys: vec![identifier],
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn child(mut self, identifier: Identifier) -> Self {
        self.keys.push(identifier);
        self
    }

    async fn build_keys(&self, registry: &dyn Registry) -> Result<Box<[usize]>, DataKeyBuildError> {
        let Some((value_identifier, registry_path)) = self.keys.split_last() else {
            return Err(DataKeyBuildError::Empty);
        };

        let mut numeric_keys = Vec::with_capacity(self.keys.len());

        build_key_path(registry, registry_path, value_identifier, &mut numeric_keys).await?;

        Ok(numeric_keys.into_boxed_slice())
    }

    pub async fn build_arc(
        self,
        registry: &Arc<dyn Registry>,
    ) -> Result<ArcDataKey<T>, DataKeyBuildError> {
        let keys = self.build_keys(registry.as_ref()).await?;

        Ok(ArcDataKey {
            keys,
            root: Arc::clone(registry),
            marker: PhantomData,
        })
    }

    pub async fn build_ref(
        self,
        registry: &dyn Registry,
    ) -> Result<RefDataKey<'_, T>, DataKeyBuildError> {
        let keys = self.build_keys(registry).await?;

        Ok(RefDataKey {
            keys,
            root: registry,
            marker: PhantomData,
        })
    }

    pub fn build_arc_blocking(
        self,
        registry: &Arc<dyn Registry>,
    ) -> Result<ArcDataKey<T>, DataKeyBuildError> {
        let keys = build_keys_blocking(&self.keys, registry.as_ref())?;

        Ok(ArcDataKey {
            keys,
            root: Arc::clone(registry),
            marker: PhantomData,
        })
    }

    pub fn build_ref_blocking(
        self,
        registry: &dyn Registry,
    ) -> Result<RefDataKey<'_, T>, DataKeyBuildError> {
        let keys = build_keys_blocking(&self.keys, registry)?;

        Ok(RefDataKey {
            keys,
            root: registry,
            marker: PhantomData,
        })
    }
}

fn build_keys_blocking(
    keys: &[Identifier],
    registry: &dyn Registry,
) -> Result<Box<[usize]>, DataKeyBuildError> {
    let Some((value_identifier, registry_path)) = keys.split_last() else {
        return Err(DataKeyBuildError::Empty);
    };

    let mut numeric_keys = Vec::with_capacity(keys.len());
    let mut current = RegistryCursor::Borrowed(registry);

    for identifier in registry_path {
        let (id, nested) = {
            let registry = current.as_ref();

            let id = registry
                .get_id(identifier)
                .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

            let value = registry
                .by_id_erased(id)
                .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

            let nested = value
                .downcast_ref::<Arc<dyn Registry>>()
                .ok_or_else(|| DataKeyBuildError::NotARegistry(identifier.clone()))?;

            let nested = Arc::clone(nested);

            drop(value);

            (id, nested)
        };

        numeric_keys.push(id);
        current = RegistryCursor::Owned(nested);
    }

    let value_id = current
        .as_ref()
        .get_id(value_identifier)
        .ok_or_else(|| DataKeyBuildError::MissingValue(value_identifier.clone()))?;

    numeric_keys.push(value_id);
    Ok(numeric_keys.into_boxed_slice())
}

fn build_key_path<'a>(
    root: &'a dyn Registry,
    registry_path: &'a [Identifier],
    value_identifier: &'a Identifier,
    numeric_keys: &'a mut Vec<usize>,
) -> BoxFuture<'a, Result<(), DataKeyBuildError>> {
    Box::pin(async move {
        let mut current = RegistryCursor::Borrowed(root);

        for identifier in registry_path {
            let (id, nested) = {
                let registry = current.as_ref();

                let id = registry
                    .get_id_async(identifier)
                    .await
                    .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

                let value = registry
                    .by_id_erased_async(id)
                    .await
                    .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

                let nested = value
                    .downcast_ref::<Arc<dyn Registry>>()
                    .ok_or_else(|| DataKeyBuildError::NotARegistry(identifier.clone()))?;

                let nested = Arc::clone(nested);

                drop(value);

                (id, nested)
            };

            numeric_keys.push(id);
            current = RegistryCursor::Owned(nested);
        }

        let value_id = current
            .as_ref()
            .get_id_async(value_identifier)
            .await
            .ok_or_else(|| DataKeyBuildError::MissingValue(value_identifier.clone()))?;

        numeric_keys.push(value_id);
        Ok(())
    })
}
