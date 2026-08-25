use crate::{
    ErasedRegistryIterator, Identifier, Registry, TypedRegistry,
    value::{DynIterator, ErasedRegistryRef},
};
use rustc_hash::FxHashMap;
use std::any::{TypeId, type_name};

/// An immutable registry holding 'static data.
pub struct StaticRegistry<T: Send + Sync + 'static> {
    static_entries: &'static [T],
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> StaticRegistry<T> {
    pub(crate) const fn new(
        static_entries: &'static [T],
        entries: Box<[T]>,
        mapping: FxHashMap<Identifier, usize>,
    ) -> Self {
        Self {
            static_entries,
            entries,
            mapping,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &T)> {
        self.mapping
            .iter()
            .filter_map(|(identifier, &id)| self.by_id(id).map(|value| (identifier, value)))
    }

    pub(crate) const fn mapping(&self) -> &FxHashMap<Identifier, usize> {
        &self.mapping
    }
}

impl<T: Send + Sync + 'static> Registry for StaticRegistry<T> {
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
        {
            if id < self.static_entries.len() {
                Some(&self.static_entries[id])
            } else {
                self.entries.get(id - self.static_entries.len())
            }
        }
        .map(|value| ErasedRegistryRef::Borrowed(value))
    }

    fn iter_erased(&self) -> ErasedRegistryIterator<'_> {
        Box::new(self.mapping.iter().filter_map(|(identifier, &id)| {
            self.by_id_erased(id)
                .map(|value| (identifier.clone(), value))
        }))
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for StaticRegistry<T> {
    type Item = &'a T;
    type IterItem = (&'a Identifier, &'a T);
    type Iter = DynIterator<'a, Self::IterItem>;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        if id < self.static_entries.len() {
            Some(&self.static_entries[id])
        } else {
            self.entries.get(id - self.static_entries.len())
        }
    }

    fn iter(&'a self) -> Self::Iter {
        DynIterator::new(
            self.mapping
                .iter()
                .filter_map(|(identifier, &id)| self.by_id(id).map(|value| (identifier, value))),
        )
    }
}
