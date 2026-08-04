use std::sync::Arc;

use crate::{Registry, RegistryAccess, error::RegistryGetError, key::DataKey};

#[derive(Clone)]
pub struct RegistryLookup(Arc<RootRegistry>);

impl RegistryLookup {
    #[must_use]
    pub fn new(root: Arc<RootRegistry>) -> Self {
        Self(root)
    }

    pub fn get<T>(&self, key: &DataKey<T>) -> Result<Arc<T>, RegistryGetError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let (item_id, registry_ids) = key.path().split_last().ok_or(RegistryGetError::EmptyPath)?;

        let mut parent = self.0.clone();

        let (registry_id, parent_ids) = registry_ids
            .split_last()
            .ok_or(RegistryGetError::EmptyPath)?;

        for identifier in parent_ids {
            let child = parent
                .get(identifier)
                .ok_or_else(|| RegistryGetError::NotFound(identifier.clone()))?;

            parent = child
                .into_any()
                .downcast::<RootRegistry>()
                .map_err(|_| RegistryGetError::ExpectedRegistry(identifier.clone()))?;
        }

        let registry = parent
            .get(registry_id)
            .ok_or_else(|| RegistryGetError::NotFound(registry_id.clone()))?;

        let expected = registry.type_name();

        let registry = registry.into_any().downcast::<Registry<T>>().map_err(|_| {
            RegistryGetError::TypeMismatch {
                identifier: registry_id.clone(),
                expected,
            }
        })?;

        registry
            .get(item_id)
            .ok_or_else(|| RegistryGetError::NotFound(item_id.clone()))
    }
}

type RootRegistry = Registry<dyn RegistryAccess + Send + Sync>;
