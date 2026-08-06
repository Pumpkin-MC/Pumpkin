use dashmap::{DashMap, Entry, mapref::multiple::RefMulti};
use pumpkin_util::{identifier::Identifier, version::MinecraftVersion};
use rustc_hash::FxHashMap;
use std::{
    any::{Any, type_name},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    RegistryAccess,
    error::{RegistryInsertError, VersionMappingError},
    mapping::{NetworkId, VersionMapping},
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
        Self {
            entries: DashMap::new(),
            version_mappings: DashMap::new(),
        }
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

        let mut mapping = mapping
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(&existing_network_id) = mapping.by_identifier.get(&identifier) {
            if existing_network_id == network_id {
                return Ok(());
            }

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

        let mut version_mapping = mapping
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

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

    #[must_use]
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

        let mapping = mapping
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        mapping.by_identifier.get(identifier).copied()
    }

    #[must_use]
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

        let mapping = mapping
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        mapping.by_network_id.get(&network_id).cloned()
    }

    #[must_use]
    pub fn get_by_network_id(
        &self,
        version: impl Into<MinecraftVersion>,
        network_id: NetworkId,
    ) -> Option<Arc<T>> {
        let identifier = self.identifier_from_network_id(version, network_id)?;

        self.get(&identifier)
    }

    #[must_use]
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

impl<T: ?Sized + Send + Sync + 'static> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Send + Sync + 'static> RegistryAccess for Registry<T> {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}
