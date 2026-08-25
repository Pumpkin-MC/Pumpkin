use crate::{
    BOOTSTRAP, RegistryConfig, error::BootstrapError, immutable::FrozenRegistry,
    mutable::ReloadableRegistry, r#static::StaticRegistry,
};
use pumpkin_util::identifier::Identifier;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::marker::PhantomData;

pub struct RegistryBuilder<T: 'static> {
    marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> RegistryBuilder<T> {
    /// Build a registry where Pumpkin's internal entries do not need to be copied to the heap.
    pub fn new_static(
        name: &Identifier,
        static_entries: &'static [T],
        identifiers: &[Identifier],
    ) -> Result<StaticRegistry<T>, BootstrapError> {
        Self::new_static_with_config(name, static_entries, identifiers, RegistryConfig::default())
    }

    pub fn new_static_with_config(
        name: &Identifier,
        static_entries: &'static [T],
        identifiers: &[Identifier],
        config: RegistryConfig,
    ) -> Result<StaticRegistry<T>, BootstrapError> {
        let statics = static_entries.len();
        assert_eq!(statics, identifiers.len());

        let (added_entries, added_mapping) =
            BOOTSTRAP
                .get()
                .ok_or(BootstrapError::Uninitialized)
                .and_then(|m| m.populate_with_config::<T>(name, config))?;

        let total = statics + added_entries.len();

        let mut mapping = FxHashMap::with_capacity_and_hasher(total, FxBuildHasher);

        // Static identifiers always come first.
        for (id, identifier) in identifiers.iter().cloned().enumerate() {
            if mapping.insert(identifier.clone(), id).is_some() && !config.allow_overwrites {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        // Bootstrap IDs need to be offset by the static entry count.
        for (identifier, id) in added_mapping {
            if mapping.insert(identifier.clone(), statics + id).is_some() {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        Ok(StaticRegistry::new(
            static_entries,
            added_entries.into_boxed_slice(),
            mapping,
        ))
    }

    /// Build a registry where all data lives on the heap.
    /// These registries may not be reloaded.
    pub fn frozen(name: &Identifier) -> Result<FrozenRegistry<T>, BootstrapError> {
        Self::frozen_with_config(name, RegistryConfig::default())
    }

    pub fn frozen_with_config(
        name: &Identifier,
        config: RegistryConfig,
    ) -> Result<FrozenRegistry<T>, BootstrapError> {
        let (entries, mapping) = BOOTSTRAP
            .get()
            .ok_or(BootstrapError::Uninitialized)
            .and_then(|m| m.populate_with_config::<T>(name, config))?;
        Ok(FrozenRegistry::new(entries.into_boxed_slice(), mapping))
    }

    /// Build a reloadable registry.
    pub fn reloadable(
        name: &Identifier,
        static_entries: &'static [T],
        identifiers: &[Identifier],
    ) -> Result<ReloadableRegistry<T>, BootstrapError> {
        Self::reloadable_with_config(name, static_entries, identifiers, RegistryConfig::default())
    }

    pub fn reloadable_with_config(
        name: &Identifier,
        static_entries: &'static [T],
        identifiers: &[Identifier],
        config: RegistryConfig,
    ) -> Result<ReloadableRegistry<T>, BootstrapError> {
        let statics = static_entries.len();
        assert_eq!(statics, identifiers.len());

        let (added_entries, added_mapping) =
            BOOTSTRAP
                .get()
                .ok_or(BootstrapError::Uninitialized)
                .and_then(|m| m.populate_with_config::<T>(name, config))?;

        let total = statics + added_entries.len();

        let mut mapping = FxHashMap::with_capacity_and_hasher(total, FxBuildHasher);

        // Static identifiers always come first.
        for (id, identifier) in identifiers.iter().cloned().enumerate() {
            if mapping.insert(identifier.clone(), id).is_some() && !config.allow_overwrites {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        // Bootstrap IDs need to be offset by the static entry count.
        for (identifier, id) in added_mapping {
            if mapping.insert(identifier.clone(), statics + id).is_some() {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        Ok(ReloadableRegistry::new(
            name.clone(),
            static_entries,
            added_entries.into_boxed_slice(),
            mapping,
            config,
        ))
    }
}
