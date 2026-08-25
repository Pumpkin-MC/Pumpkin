use crate::{
    BOOTSTRAP, ErasedRegistryIterator, Registry, RegistryConfig, StaticRegistry, TypedRegistry,
    error::BootstrapError,
    value::{DynIterator, ErasedRegistryRef, SnapshotRef},
};
use arc_swap::ArcSwap;
use pumpkin_util::identifier::Identifier;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    any::{TypeId, type_name},
    marker::PhantomData,
    sync::Arc,
};

pub struct ReloadableRegistry<T: Send + Sync + 'static> {
    inner: ArcSwap<StaticRegistry<T>>,
    name: Identifier,
    config: RegistryConfig,
}

impl<T: Send + Sync + 'static> ReloadableRegistry<T> {
    pub(crate) fn new(
        name: Identifier,
        static_entries: &'static [T],
        entries: Box<[T]>,
        mapping: FxHashMap<Identifier, usize>,
        config: RegistryConfig,
    ) -> Self {
        Self {
            inner: ArcSwap::from_pointee(StaticRegistry::new(static_entries, entries, mapping)),
            name,
            config,
        }
    }

    pub fn reload(&self) -> Result<(), BootstrapError> {
        let (entries, mapping) = BOOTSTRAP
            .get()
            .ok_or(BootstrapError::Uninitialized)
            .and_then(|manager| manager.populate_with_config::<T>(&self.name, self.config))?;
        let replacement = StaticRegistry::new(&[], entries.into_boxed_slice(), mapping);
        self.inner.store(Arc::new(replacement));
        Ok(())
    }

    /// Atomically replaces all entries while preserving caller-provided order.
    ///
    /// If validation fails, the existing registry snapshot remains unchanged.
    pub fn replace_entries<I>(&self, entries: I) -> Result<(), BootstrapError>
    where
        I: IntoIterator<Item = (Identifier, T)>,
    {
        let entries: Vec<_> = entries.into_iter().collect();
        let mut values = Vec::with_capacity(entries.len());
        let mut mapping = FxHashMap::with_capacity_and_hasher(entries.len(), FxBuildHasher);

        for (identifier, value) in entries {
            let id = values.len();
            if mapping.insert(identifier.clone(), id).is_some() {
                return Err(BootstrapError::DuplicateEntry {
                    registry: self.name.clone(),
                    identifier,
                });
            }
            values.push(value);
        }

        let replacement = StaticRegistry::new(&[], values.into_boxed_slice(), mapping);
        self.inner.store(Arc::new(replacement));
        Ok(())
    }
}

impl<T: Send + Sync + 'static> Registry for ReloadableRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.inner.load().get_id(identifier)
    }

    fn by_id_erased(&self, id: usize) -> Option<ErasedRegistryRef<'_>> {
        ErasedRegistryRef::from_snapshot(self.inner.load_full(), id)
    }

    fn iter_erased(&self) -> ErasedRegistryIterator<'_> {
        Box::new(ErasedSnapshotIterator::new(self.inner.load_full()))
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for ReloadableRegistry<T> {
    type Item = SnapshotRef<T>;
    type IterItem = (Identifier, SnapshotRef<T>);
    type Iter = DynIterator<'a, Self::IterItem>;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        SnapshotRef::new(self.inner.load_full(), id)
    }

    fn iter(&'a self) -> Self::Iter {
        DynIterator::new(SnapshotIterator::new(self.inner.load_full()))
    }
}

struct SnapshotIterator<T>
where
    T: Send + Sync + 'static,
{
    snapshot: Arc<StaticRegistry<T>>,
    entries: std::vec::IntoIter<(Identifier, usize)>,
}

impl<T> SnapshotIterator<T>
where
    T: Send + Sync + 'static,
{
    fn new(snapshot: Arc<StaticRegistry<T>>) -> Self {
        let entries = snapshot
            .mapping()
            .iter()
            .map(|(identifier, &id)| (identifier.clone(), id))
            .collect::<Vec<_>>()
            .into_iter();

        Self { snapshot, entries }
    }
}

impl<T> Iterator for SnapshotIterator<T>
where
    T: Send + Sync + 'static,
{
    type Item = (Identifier, SnapshotRef<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let (identifier, id) = self.entries.next()?;
        let value = SnapshotRef::new(Arc::clone(&self.snapshot), id)?;
        Some((identifier, value))
    }
}

struct ErasedSnapshotIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    snapshot: Arc<StaticRegistry<T>>,
    entries: std::vec::IntoIter<(Identifier, usize)>,
    marker: PhantomData<&'a ()>,
}

impl<T> ErasedSnapshotIterator<'_, T>
where
    T: Send + Sync + 'static,
{
    fn new(snapshot: Arc<StaticRegistry<T>>) -> Self {
        let entries = snapshot
            .mapping()
            .iter()
            .map(|(identifier, &id)| (identifier.clone(), id))
            .collect::<Vec<_>>()
            .into_iter();

        Self {
            snapshot,
            entries,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ErasedSnapshotIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    type Item = (Identifier, ErasedRegistryRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (identifier, id) = self.entries.next()?;
        let value = ErasedRegistryRef::from_snapshot(Arc::clone(&self.snapshot), id)?;
        Some((identifier, value))
    }
}
