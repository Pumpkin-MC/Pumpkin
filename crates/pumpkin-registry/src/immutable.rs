use std::any::{TypeId, type_name};

use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

use crate::{
    BoxFuture, Registry, TypedRegistry,
    value::{DynIterator, ErasedRegistryRef},
};

/// An immutable registry holding heap-allocated data.
pub struct FrozenRegistry<T: Send + Sync + 'static> {
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> FrozenRegistry<T> {
    pub(crate) const fn new(entries: Box<[T]>, mapping: FxHashMap<Identifier, usize>) -> Self {
        Self { entries, mapping }
    }
}

impl<T: Send + Sync + 'static> Registry for FrozenRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.mapping.get(identifier).copied()
    }

    fn get_id_async<'a>(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<usize>> {
        Box::pin(async move { self.get_id(identifier) })
    }

    fn by_id_erased(&self, id: usize) -> Option<ErasedRegistryRef<'_>> {
        self.entries
            .get(id)
            .map(|value| ErasedRegistryRef::Borrowed(value))
    }

    fn by_id_erased_async(&self, id: usize) -> BoxFuture<'_, Option<ErasedRegistryRef<'_>>> {
        Box::pin(async move { self.by_id_erased(id) })
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for FrozenRegistry<T> {
    type Item = &'a T;
    type IterItem = (&'a Identifier, &'a T);
    type Iter = DynIterator<'a, Self::IterItem>;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        self.entries.get(id)
    }

    fn iter(&'a self) -> Self::Iter {
        DynIterator::new(
            self.mapping
                .iter()
                .filter_map(|(identifier, &id)| self.by_id(id).map(|value| (identifier, value))),
        )
    }
}
