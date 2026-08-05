use crate::{error::RegistryInsertError, registry::Registry};
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;
use std::{
    iter::Chain,
    slice::{Iter, IterMut},
};

pub struct RegistryBuilder<T: Send + Sync + 'static> {
    static_entries: &'static [T],
    entries: Vec<T>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> RegistryBuilder<T> {
    pub fn new(static_entries: &'static [T]) -> Self {
        Self {
            static_entries,
            entries: Vec::new(),
            mapping: FxHashMap::default(),
        }
    }

    pub fn register(
        &mut self,
        identifier: Identifier,
        value: T,
    ) -> Result<(), RegistryInsertError> {
        if self.mapping.contains_key(&identifier) {
            return Err(RegistryInsertError::AlreadyRegistered(identifier));
        }

        let id = self.entries.len();
        self.entries.push(value);
        self.mapping
            .insert(identifier, id + self.static_entries.len());
        Ok(())
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
        if id >= self.len() {
            return None;
        }

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

    pub fn iter(&self) -> Chain<Iter<'_, T>, Iter<'_, T>> {
        self.static_entries.iter().chain(self.entries.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.entries.iter_mut()
    }

    pub(crate) fn build(self) -> Registry<T> {
        Registry::new(
            self.static_entries,
            self.entries.into_boxed_slice(),
            self.mapping,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Identifier {
        Identifier::parse_static(value)
    }

    #[test]
    fn registers_entries_after_static_entries() {
        static STATIC: [u32; 2] = [10, 20];
        let mut builder = RegistryBuilder::new(&STATIC);

        builder.register(id("test:first"), 30).unwrap();
        builder.register(id("test:second"), 40).unwrap();

        assert_eq!(builder.len(), 4);
        assert_eq!(builder.get_id(&id("test:first")), Some(2));
        assert_eq!(builder.get_id(&id("test:second")), Some(3));
        assert_eq!(builder.get_by_id(0), Some(&10));
        assert_eq!(builder.get_by_id(1), Some(&20));
        assert_eq!(builder.get_by_id(2), Some(&30));
        assert_eq!(builder.get_by_id(3), Some(&40));
        assert_eq!(builder.get_by_id(4), None);
    }

    #[test]
    fn rejects_duplicate_identifiers_without_inserting_value() {
        let mut builder = RegistryBuilder::new(&[]);
        let identifier = id("test:value");

        builder.register(identifier.clone(), 1u32).unwrap();
        let error = builder.register(identifier.clone(), 2u32).unwrap_err();

        assert!(matches!(
            error,
            RegistryInsertError::AlreadyRegistered(found) if found == identifier
        ));
        assert_eq!(builder.len(), 1);
        assert_eq!(builder.get(&identifier), Some(&1));
    }

    #[test]
    fn iter_mut_only_changes_dynamic_entries() {
        static STATIC: [u32; 1] = [5];
        let mut builder = RegistryBuilder::new(&STATIC);
        builder.register(id("test:value"), 10).unwrap();

        for value in builder.iter_mut() {
            *value += 1;
        }

        assert_eq!(builder.iter().copied().collect::<Vec<_>>(), vec![5, 11]);
    }

    #[test]
    fn build_preserves_ids_values_and_iteration_order() {
        static STATIC: [u32; 1] = [7];
        let mut builder = RegistryBuilder::new(&STATIC);
        let first = id("test:first");
        let second = id("test:second");
        builder.register(first.clone(), 8).unwrap();
        builder.register(second.clone(), 9).unwrap();

        let registry = builder.build();

        assert_eq!(registry.get_id(&first), Some(1));
        assert_eq!(registry.get_id(&second), Some(2));
        assert_eq!(registry.get(&first), Some(&8));
        assert_eq!(registry.iter().copied().collect::<Vec<_>>(), vec![7, 8, 9]);
    }
}
