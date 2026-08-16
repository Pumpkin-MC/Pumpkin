use crate::{
    BOOTSTRAP, ErasedRegistryIterator, Registry, TypedRegistry,
    error::BootstrapError,
    immutable::FrozenRegistry,
    value::{DynIterator, ErasedRegistryRef, SnapshotRef},
};
use arc_swap::ArcSwap;
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;
use std::{
    any::{TypeId, type_name},
    marker::PhantomData,
    sync::Arc,
};

pub struct ReloadableRegistry<T: Send + Sync + 'static> {
    inner: ArcSwap<FrozenRegistry<T>>,
    name: Identifier,
}

impl<T: Send + Sync + 'static> ReloadableRegistry<T> {
    pub(crate) fn new(
        name: Identifier,
        entries: Box<[T]>,
        mapping: FxHashMap<Identifier, usize>,
    ) -> Self {
        Self {
            inner: ArcSwap::from_pointee(FrozenRegistry::new(entries, mapping)),
            name,
        }
    }

    pub fn reload(&self) -> Result<(), BootstrapError> {
        let (entries, mapping) = BOOTSTRAP
            .get()
            .ok_or(BootstrapError::Uninitialized)
            .and_then(|manager| manager.populate::<T>(&self.name))?;
        let replacement = FrozenRegistry::new(entries.into_boxed_slice(), mapping);
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
    snapshot: Arc<FrozenRegistry<T>>,
    next_id: usize,
}

impl<T> SnapshotIterator<T>
where
    T: Send + Sync + 'static,
{
    const fn new(snapshot: Arc<FrozenRegistry<T>>) -> Self {
        Self {
            snapshot,
            next_id: 0,
        }
    }
}

impl<T> Iterator for SnapshotIterator<T>
where
    T: Send + Sync + 'static,
{
    type Item = (Identifier, SnapshotRef<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next_id;
        let identifier = self.snapshot.identifier_by_id(id)?.clone();
        let value = SnapshotRef::new(Arc::clone(&self.snapshot), id)?;
        self.next_id += 1;
        Some((identifier, value))
    }
}

struct ErasedSnapshotIterator<'a, T>
where
    T: Send + Sync + 'static,
{
    snapshot: Arc<FrozenRegistry<T>>,
    next_id: usize,
    marker: PhantomData<&'a ()>,
}

impl<T> ErasedSnapshotIterator<'_, T>
where
    T: Send + Sync + 'static,
{
    const fn new(snapshot: Arc<FrozenRegistry<T>>) -> Self {
        Self {
            snapshot,
            next_id: 0,
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
        let id = self.next_id;
        let identifier = self.snapshot.identifier_by_id(id)?.clone();
        let value = ErasedRegistryRef::from_snapshot(Arc::clone(&self.snapshot), id)?;
        self.next_id += 1;
        Some((identifier, value))
    }
}
