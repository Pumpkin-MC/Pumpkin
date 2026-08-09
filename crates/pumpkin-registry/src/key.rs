use crate::{
    BoxFuture, BoxedRegistry, Registry,
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

        let mut guards = Vec::with_capacity(keys.len());

        // This pointer either points at `root`, which lives for `'a`,
        // or at a registry kept alive by one of `guards`.
        let mut current: *const dyn Registry = root;

        for &registry_id in registry_path {
            // SAFETY:
            //
            // `current` either:
            // 1. points to `root`, which lives for `'a`, or
            // 2. points into one of the guards already stored in `guards`.
            //
            // We never remove guards while traversing.
            let registry = unsafe { &*current };

            let value = registry
                .get_by_id(registry_id)
                .await
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

            let nested = value
                .downcast_ref::<BoxedRegistry>()
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

            current = ptr::from_ref::<dyn Registry>(nested.as_ref());

            // Keep the storage containing `nested` alive.
            guards.push(value);
        }

        // SAFETY: same invariant as above.
        let registry = unsafe { &*current };

        let value = registry
            .get_by_id(value_id)
            .await
            .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

        let typed = value
            .downcast_ref::<T>()
            .ok_or(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: registry.item_type_name(),
            })?;

        let value_ptr = ptr::from_ref::<T>(typed);

        guards.push(value);

        Ok(DataKeyRef {
            _guards: guards,
            value: value_ptr,
            marker: PhantomData,
        })
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

    let mut guards = Vec::with_capacity(keys.len());
    let mut current: *const dyn Registry = root;

    for &registry_id in registry_path {
        // SAFETY: `current` points either to `root` or into storage kept alive
        // by one of the guards retained in `guards`.
        let registry = unsafe { &*current };

        let value = registry
            .get_by_id_blocking(registry_id)
            .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

        let nested = value
            .downcast_ref::<BoxedRegistry>()
            .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;

        current = ptr::from_ref::<dyn Registry>(nested.as_ref());
        guards.push(value);
    }

    // SAFETY: same invariant as above.
    let registry = unsafe { &*current };

    let value = registry
        .get_by_id_blocking(value_id)
        .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

    let typed = value
        .downcast_ref::<T>()
        .ok_or(DataKeyGetError::TypeMismatch {
            expected: type_name::<T>(),
            actual: registry.item_type_name(),
        })?;

    let value_ptr = ptr::from_ref::<T>(typed);
    guards.push(value);

    Ok(DataKeyRef {
        _guards: guards,
        value: value_ptr,
        marker: PhantomData,
    })
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

    #[allow(clippy::collection_is_never_read)]
    let mut guards = Vec::with_capacity(registry_path.len());

    let mut current: *const dyn Registry = registry;

    for identifier in registry_path {
        // SAFETY: `current` points either to `registry` or into a value kept
        // alive by a guard retained in `guards`.
        let registry = unsafe { &*current };

        let id = registry
            .get_id_blocking(identifier)
            .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

        let value = registry
            .get_by_id_blocking(id)
            .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

        let nested = value
            .downcast_ref::<BoxedRegistry>()
            .ok_or_else(|| DataKeyBuildError::NotARegistry(identifier.clone()))?;

        numeric_keys.push(id);
        current = ptr::from_ref::<dyn Registry>(nested.as_ref());
        guards.push(value);
    }

    // SAFETY: same invariant as above.
    let registry = unsafe { &*current };
    let value_id = registry
        .get_id_blocking(value_identifier)
        .ok_or_else(|| DataKeyBuildError::MissingValue(value_identifier.clone()))?;

    numeric_keys.push(value_id);
    Ok(numeric_keys.into_boxed_slice())
}

fn build_key_path<'a>(
    current: &'a dyn Registry,
    registry_path: &'a [Identifier],
    value_identifier: &'a Identifier,
    numeric_keys: &'a mut Vec<usize>,
) -> BoxFuture<'a, Result<(), DataKeyBuildError>> {
    Box::pin(async move {
        let Some((identifier, remaining_path)) = registry_path.split_first() else {
            let value_id = current
                .get_id(value_identifier)
                .await
                .ok_or_else(|| DataKeyBuildError::MissingValue(value_identifier.clone()))?;

            numeric_keys.push(value_id);
            return Ok(());
        };

        let id = current
            .get_id(identifier)
            .await
            .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

        let registry = current
            .get_by_id(id)
            .await
            .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

        let registry = registry
            .downcast_ref::<BoxedRegistry>()
            .ok_or_else(|| DataKeyBuildError::NotARegistry(identifier.clone()))?;

        numeric_keys.push(id);

        build_key_path(
            registry.as_ref(),
            remaining_path,
            value_identifier,
            numeric_keys,
        )
        .await
    })
}
