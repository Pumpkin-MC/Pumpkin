use dashmap::{DashMap, Entry, mapref::multiple::RefMulti};
use pumpkin_util::{identifier::Identifier, version::MinecraftVersion};
use rustc_hash::FxHashMap;
use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock},
};

use crate::{
    RegistryAccess, RegistryLookup, error::{RegistryInsertError, VersionMappingError}, mapping::{NetworkId, VersionMapping},
};

pub struct Registry<T: ?Sized + Send + Sync + 'static> {
    entries: DashMap<Identifier, Arc<T>>,
    version_mappings: DashMap<MinecraftVersion, Arc<RwLock<VersionMapping>>>,
}

impl<T: Send + Sync + 'static> Registry<T> {
    pub fn register(&self, identifier: Identifier, value: T) -> Result<(), RegistryInsertError> {
        self.register_arc(identifier, Arc::new(value))
    }

    pub fn get_or_register(&self, identifier: Identifier, create: impl FnOnce() -> T) -> Arc<T> {
        match self.entries.entry(identifier) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let value = Arc::new(create());
                entry.insert(Arc::clone(&value));
                value
            }
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> Registry<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_arc(
        &self,
        identifier: Identifier,
        value: Arc<T>,
    ) -> Result<(), RegistryInsertError> {
        match self.entries.entry(identifier) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
            Entry::Occupied(entry) => {
                Err(RegistryInsertError::AlreadyRegistered(entry.key().clone()))
            }
        }
    }

    pub fn get_or_register_arc(
        &self,
        identifier: Identifier,
        create: impl FnOnce() -> Arc<T>,
    ) -> Arc<T> {
        match self.entries.entry(identifier) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let value = create();
                entry.insert(Arc::clone(&value));
                value
            }
        }
    }

    pub fn register_version_mapping(
        &self,
        version: impl Into<MinecraftVersion>,
        identifier: Identifier,
        network_id: NetworkId,
    ) -> Result<(), VersionMappingError> {
        let version = version.into();

        if !self.entries.contains_key(&identifier) {
            return Err(VersionMappingError::UnknownEntry(identifier));
        }

        let mapping = {
            let entry = self
                .version_mappings
                .entry(version)
                .or_insert_with(|| Arc::new(RwLock::new(VersionMapping::new())));

            Arc::clone(entry.value())
        };

        // The DashMap entry guard has been dropped before awaiting.
        let mut mapping = mapping.write().unwrap_or_else(PoisonError::into_inner);

        if let Some(&existing_network_id) = mapping.by_identifier.get(&identifier)
            && existing_network_id != network_id
        {
            return Err(VersionMappingError::IdentifierAlreadyMapped {
                version,
                identifier,
                existing_network_id,
                requested_network_id: network_id,
            });
        }

        if let Some(existing_identifier) = mapping.by_network_id.get(&network_id) {
            return Err(VersionMappingError::NetworkIdAlreadyMapped {
                version,
                network_id,
                existing_identifier: existing_identifier.clone(),
                requested_identifier: identifier,
            });
        }

        mapping.by_identifier.insert(identifier.clone(), network_id);
        mapping.by_network_id.insert(network_id, identifier);

        Ok(())
    }

    pub fn register_version_mappings<I>(
        &self,
        version: impl Into<MinecraftVersion>,
        mappings: I,
    ) -> Result<(), VersionMappingError>
    where
        I: IntoIterator<Item = (Identifier, NetworkId)>,
    {
        let version = version.into();
        let mappings: Vec<_> = mappings.into_iter().collect();

        let mapping = {
            let entry = self
                .version_mappings
                .entry(version)
                .or_insert_with(|| Arc::new(RwLock::new(VersionMapping::new())));

            Arc::clone(entry.value())
        };

        let mut version_mapping = mapping.write().unwrap_or_else(PoisonError::into_inner);

        let mut batch_by_identifier: HashMap<Identifier, u32, _> = FxHashMap::default();
        let mut batch_by_network_id: HashMap<u32, Identifier, _> = FxHashMap::default();

        for (identifier, network_id) in &mappings {
            if !self.entries.contains_key(identifier) {
                return Err(VersionMappingError::UnknownEntry(identifier.clone()));
            }

            if let Some(&existing_network_id) = version_mapping.by_identifier.get(identifier)
                && existing_network_id != *network_id
            {
                return Err(VersionMappingError::IdentifierAlreadyMapped {
                    version,
                    identifier: identifier.clone(),
                    existing_network_id,
                    requested_network_id: *network_id,
                });
            }

            if let Some(existing_identifier) = version_mapping.by_network_id.get(network_id)
                && existing_identifier != identifier
            {
                return Err(VersionMappingError::NetworkIdAlreadyMapped {
                    version,
                    network_id: *network_id,
                    existing_identifier: existing_identifier.clone(),
                    requested_identifier: identifier.clone(),
                });
            }

            if let Some(&existing_network_id) = batch_by_identifier.get(identifier)
                && existing_network_id != *network_id
            {
                return Err(VersionMappingError::IdentifierAlreadyMapped {
                    version,
                    identifier: identifier.clone(),
                    existing_network_id,
                    requested_network_id: *network_id,
                });
            }

            if let Some(existing_identifier) = batch_by_network_id.get(network_id)
                && existing_identifier != identifier
            {
                return Err(VersionMappingError::NetworkIdAlreadyMapped {
                    version,
                    network_id: *network_id,
                    existing_identifier: existing_identifier.clone(),
                    requested_identifier: identifier.clone(),
                });
            }

            batch_by_identifier.insert(identifier.clone(), *network_id);
            batch_by_network_id.insert(*network_id, identifier.clone());
        }

        for (identifier, network_id) in mappings {
            version_mapping
                .by_identifier
                .insert(identifier.clone(), network_id);

            version_mapping.by_network_id.insert(network_id, identifier);
        }

        Ok(())
    }

    pub fn network_id(
        &self,
        version: impl Into<MinecraftVersion>,
        identifier: &Identifier,
    ) -> Option<NetworkId> {
        let version = version.into();

        let mapping = {
            let mapping = self.version_mappings.get(&version)?;
            Arc::clone(mapping.value())
        };

        let mapping = mapping.read().unwrap_or_else(PoisonError::into_inner);

        mapping.by_identifier.get(identifier).copied()
    }

    pub fn identifier_from_network_id(
        &self,
        version: impl Into<MinecraftVersion>,
        network_id: NetworkId,
    ) -> Option<Identifier> {
        let version = version.into();

        let mapping = {
            let mapping = self.version_mappings.get(&version)?;
            Arc::clone(mapping.value())
        };

        let mapping = mapping.read().unwrap_or_else(PoisonError::into_inner);

        mapping.by_network_id.get(&network_id).cloned()
    }

    pub fn get_by_network_id(
        &self,
        version: impl Into<MinecraftVersion>,
        network_id: NetworkId,
    ) -> Option<Arc<T>> {
        let identifier = self.identifier_from_network_id(version, network_id)?;

        self.get(&identifier)
    }

    pub fn has_version_mapping(&self, version: impl Into<MinecraftVersion>) -> bool {
        self.version_mappings.contains_key(&version.into())
    }

    #[must_use]
    pub fn get(&self, identifier: &Identifier) -> Option<Arc<T>> {
        self.entries
            .get(identifier)
            .map(|entry| (*entry.value()).clone())
    }

    #[must_use]
    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.entries.contains_key(identifier)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, Identifier, Arc<T>>> {
        self.entries.iter()
    }

    #[must_use]
    pub fn remove(&self, identifier: &Identifier) -> Option<Arc<T>> {
        self.entries.remove(identifier).map(|(_, value)| value)
    }
}

impl<T: ?Sized + Send + Sync + 'static> RegistryAccess for Registry<T> {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn type_id(&self) -> std::any::TypeId {
        TypeId::of::<T>()
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

impl<T: ?Sized + Send + Sync + 'static> RegistryLookup for Registry<T> {}

impl<T: ?Sized + Send + Sync + 'static> Default for Registry<T> {
    fn default() -> Self {
        Self {
            entries: DashMap::new(),
            version_mappings: DashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            Arc, Barrier,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    fn id(value: &'static str) -> Identifier {
        Identifier::from_static("test", value)
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = Registry::<u32>::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains(&id("missing")));
        assert!(registry.get(&id("missing")).is_none());
    }

    #[test]
    fn register_stores_and_returns_value() {
        let registry = Registry::new();
        registry.register(id("answer"), 42u32).unwrap();

        assert_eq!(*registry.get(&id("answer")).unwrap(), 42);
        assert!(registry.contains(&id("answer")));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn register_arc_preserves_arc_identity() {
        let registry = Registry::new();
        let value = Arc::new(String::from("shared"));

        registry
            .register_arc(id("value"), Arc::clone(&value))
            .unwrap();
        let stored = registry.get(&id("value")).unwrap();

        assert!(Arc::ptr_eq(&value, &stored));
    }

    #[test]
    fn duplicate_registration_keeps_original_value() {
        let registry = Registry::new();
        registry.register(id("value"), 1u32).unwrap();

        let error = registry.register(id("value"), 2u32).unwrap_err();

        assert!(
            matches!(error, RegistryInsertError::AlreadyRegistered(identifier) if identifier == id("value"))
        );
        assert_eq!(*registry.get(&id("value")).unwrap(), 1);
    }

    #[test]
    fn get_or_register_only_calls_factory_for_missing_entry() {
        let registry = Registry::new();
        registry.register(id("value"), 7u32).unwrap();
        let calls = AtomicUsize::new(0);

        let value = registry.get_or_register(id("value"), || {
            calls.fetch_add(1, Ordering::Relaxed);
            99
        });

        assert_eq!(*value, 7);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn get_or_register_arc_inserts_factory_result() {
        let registry = Registry::new();
        let created = Arc::new(String::from("created"));

        let returned = registry.get_or_register_arc(id("value"), || Arc::clone(&created));

        assert!(Arc::ptr_eq(&created, &returned));
        assert!(Arc::ptr_eq(&created, &registry.get(&id("value")).unwrap()));
    }

    #[test]
    fn remove_returns_value_and_updates_registry() {
        let registry = Registry::new();
        registry.register(id("value"), 11u32).unwrap();

        let removed = registry.remove(&id("value")).unwrap();

        assert_eq!(*removed, 11);
        assert!(registry.is_empty());
        assert!(registry.remove(&id("value")).is_none());
    }

    #[test]
    fn iter_visits_every_registered_entry() {
        let registry = Registry::new();
        registry.register(id("one"), 1u32).unwrap();
        registry.register(id("two"), 2u32).unwrap();
        registry.register(id("three"), 3u32).unwrap();

        let mut values: Vec<_> = registry.iter().map(|entry| **entry.value()).collect();
        values.sort_unstable();

        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn concurrent_get_or_register_creates_exactly_one_entry() {
        const THREADS: usize = 16;
        let registry = Arc::new(Registry::new());
        let barrier = Arc::new(Barrier::new(THREADS));
        let factory_calls = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();

        for _ in 0..THREADS {
            let registry = Arc::clone(&registry);
            let barrier = Arc::clone(&barrier);
            let factory_calls = Arc::clone(&factory_calls);
            handles.push(thread::spawn(move || {
                barrier.wait();
                registry.get_or_register(id("shared"), || {
                    factory_calls.fetch_add(1, Ordering::SeqCst);
                    123u32
                })
            }));
        }

        let values: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
        assert_eq!(registry.len(), 1);
        assert!(values.iter().all(|value| Arc::ptr_eq(value, &values[0])));
    }

    #[test]
    fn concurrent_registration_has_one_winner() {
        const THREADS: usize = 12;
        let registry = Arc::new(Registry::new());
        let barrier = Arc::new(Barrier::new(THREADS));

        let mut handles = Vec::new();

        for index in 0..THREADS {
            let registry = Arc::clone(&registry);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                barrier.wait();
                registry.register(id("shared"), index)
            }));
        }

        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results.iter().filter(|result| result.is_err()).count(),
            THREADS - 1
        );
        assert_eq!(registry.len(), 1);
    }
}
