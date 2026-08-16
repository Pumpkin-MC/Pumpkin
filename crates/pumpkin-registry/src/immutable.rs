use std::any::{TypeId, type_name};

use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

use crate::{
    ErasedRegistryIterator, Registry, TypedRegistry,
    value::{DynIterator, ErasedRegistryRef},
};

/// An immutable registry holding heap-allocated data.
pub struct FrozenRegistry<T: Send + Sync + 'static> {
    entries: Box<[T]>,
    identifiers: Box<[Identifier]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> FrozenRegistry<T> {
    pub(crate) fn new(entries: Box<[T]>, mapping: FxHashMap<Identifier, usize>) -> Self {
        let mut identifiers: Vec<_> = mapping
            .iter()
            .map(|(identifier, &id)| (id, identifier.clone()))
            .collect();
        identifiers.sort_unstable_by_key(|(id, _)| *id);
        let identifiers = identifiers
            .into_iter()
            .map(|(_, identifier)| identifier)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            entries,
            identifiers,
            mapping,
        }
    }

    pub(crate) fn identifier_by_id(&self, id: usize) -> Option<&Identifier> {
        self.identifiers.get(id)
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

    fn by_id_erased(&self, id: usize) -> Option<ErasedRegistryRef<'_>> {
        self.entries
            .get(id)
            .map(|value| ErasedRegistryRef::Borrowed(value))
    }

    fn iter_erased(&self) -> ErasedRegistryIterator<'_> {
        Box::new(
            self.identifiers
                .iter()
                .zip(self.entries.iter())
                .map(|(identifier, value)| {
                    (identifier.clone(), ErasedRegistryRef::Borrowed(value))
                }),
        )
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
        DynIterator::new(self.identifiers.iter().zip(self.entries.iter()))
    }
}
