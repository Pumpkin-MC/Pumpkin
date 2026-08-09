use std::any::{TypeId, type_name};

use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

use crate::{
    BoxFuture, BoxedRegistry, MutableRegistry, Registry,
    builder::RegistryBuilder,
    value::{ErasedRegistryRef, RegistryRef},
};

pub struct ImmutableRegistry<T: Send + Sync + 'static> {
    static_entries: &'static [T],
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> ImmutableRegistry<T> {
    pub const fn new(
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

    #[must_use]
    pub fn get(&self, identifier: &Identifier) -> Option<&T> {
        self.get_id(identifier).and_then(|id| {
            if id < self.static_entries.len() {
                Some(&self.static_entries[id])
            } else {
                self.entries.get(id - self.static_entries.len())
            }
        })
    }

    #[must_use]
    pub fn get_by_id(&self, id: usize) -> Option<&T> {
        if id < self.static_entries.len() {
            Some(&self.static_entries[id])
        } else {
            self.entries.get(id - self.static_entries.len())
        }
    }

    #[must_use]
    pub fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.mapping.get(identifier).copied()
    }

    #[must_use]
    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.mapping.contains_key(identifier)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len() + self.static_entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.static_entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &T)> {
        self.mapping.iter().filter_map(|(identifier, &index)| {
            self.get_by_id(index).map(|value| (identifier, value))
        })
    }
}

impl<T: Send + Sync + 'static> Registry for ImmutableRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id<'a>(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<usize>> {
        Box::pin(async move { Self::get_id(self, identifier) })
    }

    fn get_by_id(&self, id: usize) -> BoxFuture<'_, Option<ErasedRegistryRef<'_>>> {
        Box::pin(async move {
            Self::get_by_id(self, id)
                .map(RegistryRef::Borrowed)
                .map(ErasedRegistryRef::new)
        })
    }

    fn get_id_blocking(&self, identifier: &Identifier) -> Option<usize> {
        Self::get_id(self, identifier)
    }

    fn get_by_id_blocking(&self, id: usize) -> Option<ErasedRegistryRef<'_>> {
        Self::get_by_id(self, id)
            .map(RegistryRef::Borrowed)
            .map(ErasedRegistryRef::new)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_immutable(self: Box<Self>) -> BoxFuture<'static, BoxedRegistry> {
        Box::pin(async move { self as BoxedRegistry })
    }
}

impl<T: Send + Sync + 'static> From<RegistryBuilder<T>> for ImmutableRegistry<T> {
    fn from(value: RegistryBuilder<T>) -> Self {
        Self {
            entries: value.entries.into_boxed_slice(),
            mapping: value.mapping,
            static_entries: value.static_entries,
        }
    }
}

impl<T: Send + Sync + 'static> From<MutableRegistry<T>> for ImmutableRegistry<T> {
    fn from(value: MutableRegistry<T>) -> Self {
        value.0.into_inner().into()
    }
}
