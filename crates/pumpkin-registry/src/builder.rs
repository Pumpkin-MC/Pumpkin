use crate::{
    BOOTSTRAP, error::BootstrapError, immutable::FrozenRegistry, mutable::ReloadableRegistry,
    r#static::StaticRegistry,
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
        let statics = static_entries.len();
        assert_eq!(statics, identifiers.len());

        let (added_entries, added_mapping) = BOOTSTRAP.populate::<T>(name)?;

        let total = statics + added_entries.len();

        let mut mapping = FxHashMap::with_capacity_and_hasher(total, FxBuildHasher);

        // Static identifiers always come first.
        for (id, identifier) in identifiers.iter().cloned().enumerate() {
            if mapping.insert(identifier.clone(), id).is_some() {
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
    pub fn frozen(
        name: &Identifier,
        internal_entries: Vec<T>,
        identifiers: &[Identifier],
    ) -> Result<FrozenRegistry<T>, BootstrapError> {
        let internals = internal_entries.len();
        assert_eq!(internal_entries.len(), identifiers.len());

        let (added_entries, added_mapping) = BOOTSTRAP.populate::<T>(name)?;

        let total = internals + added_entries.len();

        let mut mapping = FxHashMap::with_capacity_and_hasher(total, FxBuildHasher);
        let mut entries = Vec::with_capacity(total);

        // Static identifiers always come first.
        for (id, identifier) in identifiers.iter().cloned().enumerate() {
            if mapping.insert(identifier.clone(), id).is_some() {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        entries.extend(internal_entries);

        // Bootstrap IDs need to be offset by the static entry count.
        for (identifier, id) in added_mapping {
            if mapping.insert(identifier.clone(), internals + id).is_some() {
                return Err(BootstrapError::DuplicateEntry {
                    registry: name.clone(),
                    identifier,
                });
            }
        }

        entries.extend(added_entries);

        Ok(FrozenRegistry::new(entries.into_boxed_slice(), mapping))
    }

    /// Build a reloadable registry.
    pub fn reloadable(name: &Identifier) -> Result<ReloadableRegistry<T>, BootstrapError> {
        let (entries, mapping) = BOOTSTRAP.populate::<T>(name)?;
        Ok(ReloadableRegistry::new(
            name.clone(),
            entries.into_boxed_slice(),
            mapping,
        ))
    }
}
