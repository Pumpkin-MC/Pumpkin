use crate::{
    LockableRegistry, NestRegistry,
    error::{DataKeyBuildError, DataKeyGetError},
};
use pumpkin_util::identifier::Identifier;
use std::{
    any::{TypeId, type_name},
    marker::PhantomData,
    sync::Arc,
};

pub struct ArcDataKey<T: Send + Sync + 'static> {
    keys: Box<[usize]>,
    root: Arc<NestRegistry>,
    marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Clone for ArcDataKey<T> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            root: self.root.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> ArcDataKey<T> {
    pub fn get(&self) -> Result<&T, DataKeyGetError> {
        let Some((&value_id, registry_path)) = self.keys.split_last() else {
            return Err(DataKeyGetError::InvalidKey);
        };

        let mut current: &dyn LockableRegistry = &*self.root;

        for &registry_id in registry_path {
            current = current
                .get_by_id(registry_id)
                .and_then(|value| value.downcast_ref::<Box<dyn LockableRegistry>>())
                .map(Box::as_ref)
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;
        }

        if LockableRegistry::type_id(current) != TypeId::of::<T>() {
            return Err(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: current.type_name(),
            });
        }

        let value = current
            .get_by_id(value_id)
            .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

        value
            .downcast_ref::<T>()
            .ok_or(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: current.type_name(),
            })
    }

    #[must_use]
    pub fn ids(&self) -> &[usize] {
        &self.keys
    }
}

pub struct RefDataKey<'a, T: Send + Sync + 'static> {
    keys: Box<[usize]>,
    root: &'a NestRegistry,
    marker: PhantomData<T>,
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

impl<T: Send + Sync + 'static> RefDataKey<'_, T> {
    pub fn get(&self) -> Result<&T, DataKeyGetError> {
        let Some((&value_id, registry_path)) = self.keys.split_last() else {
            return Err(DataKeyGetError::InvalidKey);
        };

        let mut current: &dyn LockableRegistry = self.root;

        for &registry_id in registry_path {
            current = current
                .get_by_id(registry_id)
                .and_then(|value| value.downcast_ref::<Box<dyn LockableRegistry>>())
                .map(Box::as_ref)
                .ok_or(DataKeyGetError::MissingRegistry { id: registry_id })?;
        }

        if LockableRegistry::type_id(current) != TypeId::of::<T>() {
            return Err(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: current.type_name(),
            });
        }

        let value = current
            .get_by_id(value_id)
            .ok_or(DataKeyGetError::MissingValue { id: value_id })?;

        value
            .downcast_ref::<T>()
            .ok_or(DataKeyGetError::TypeMismatch {
                expected: type_name::<T>(),
                actual: current.type_name(),
            })
    }

    #[must_use]
    pub fn ids(&self) -> &[usize] {
        &self.keys
    }
}

pub struct DataKeyBuilder<T: Send + Sync + 'static> {
    keys: Vec<Identifier>,
    marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> DataKeyBuilder<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keys: Vec::new(),
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn child(mut self, identifier: Identifier) -> Self {
        self.keys.push(identifier);
        self
    }

    fn build_keys(&self, registry: &NestRegistry) -> Result<Box<[usize]>, DataKeyBuildError> {
        let Some((value_identifier, registry_path)) = self.keys.split_last() else {
            return Err(DataKeyBuildError::Empty);
        };

        let mut current: &dyn LockableRegistry = registry;
        let mut numeric_keys = Vec::with_capacity(self.keys.len());

        for identifier in registry_path {
            let id = current
                .get_id(identifier)
                .ok_or_else(|| DataKeyBuildError::MissingRegistry(identifier.clone()))?;

            numeric_keys.push(id);

            current = current
                .get_by_id(id)
                .and_then(|value| value.downcast_ref::<Box<dyn LockableRegistry>>())
                .map(Box::as_ref)
                .ok_or_else(|| DataKeyBuildError::NotARegistry(identifier.clone()))?;
        }

        if LockableRegistry::type_id(current) != TypeId::of::<T>() {
            return Err(DataKeyBuildError::TypeMismatch {
                expected: type_name::<T>(),
                actual: current.type_name(),
            });
        }

        let value_id = current
            .get_id(value_identifier)
            .ok_or_else(|| DataKeyBuildError::MissingValue(value_identifier.clone()))?;

        numeric_keys.push(value_id);

        Ok(numeric_keys.into_boxed_slice())
    }

    pub fn build_arc(
        self,
        registry: &Arc<NestRegistry>,
    ) -> Result<ArcDataKey<T>, DataKeyBuildError> {
        let keys = self.build_keys(registry.as_ref())?;

        Ok(ArcDataKey {
            keys,
            root: Arc::clone(registry),
            marker: PhantomData,
        })
    }

    pub fn build_ref(
        self,
        registry: &NestRegistry,
    ) -> Result<RefDataKey<'_, T>, DataKeyBuildError> {
        let keys = self.build_keys(registry)?;

        Ok(RefDataKey {
            keys,
            root: registry,
            marker: PhantomData,
        })
    }
}

impl<T: Send + Sync + 'static> Default for DataKeyBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
