use std::sync::Arc;

use crate::{Registry, RegistryAccess, error::RegistryGetError, key::DataKey};

#[derive(Clone)]
pub struct RegistryLookup(Arc<RootRegistry>);

impl RegistryLookup {
    #[must_use]
    pub fn new(root: Arc<RootRegistry>) -> Self {
        Self(root)
    }

    pub fn get<T>(&self, key: &DataKey<T>) -> Result<Arc<Registry<T>>, RegistryGetError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let (registry_id, parent_ids) =
            key.path().split_last().ok_or(RegistryGetError::EmptyPath)?;

        let mut parent = self.0.clone();

        for identifier in parent_ids {
            let erased = parent
                .get(identifier)
                .ok_or_else(|| RegistryGetError::NotFound(identifier.clone()))?;

            parent = erased
                .into_any()
                .downcast::<RootRegistry>()
                .map_err(|_| RegistryGetError::ExpectedRegistry(identifier.clone()))?;
        }

        let erased = parent
            .get(registry_id)
            .ok_or_else(|| RegistryGetError::NotFound(registry_id.clone()))?;

        let expected = erased.type_name();

        erased
            .into_any()
            .downcast::<Registry<T>>()
            .map_err(|_| RegistryGetError::TypeMismatch {
                identifier: registry_id.clone(),
                expected,
            })
    }
}

type RootRegistry = Registry<dyn RegistryAccess + Send + Sync>;
