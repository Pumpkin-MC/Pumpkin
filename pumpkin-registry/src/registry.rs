use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;
use std::{iter::Chain, slice::Iter};

pub struct Registry<T: Send + Sync + 'static> {
    static_entries: &'static [T],
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> Registry<T> {
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

    pub fn get_mut(&mut self, identifier: &Identifier) -> Option<&mut T> {
        let id = self.get_id(identifier)?;
        self.get_by_id_mut(id)
    }

    pub fn get_by_id_mut(&mut self, id: usize) -> Option<&mut T> {
        let dynamic_id = id.checked_sub(self.static_entries.len())?;
        self.entries.get_mut(dynamic_id)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Identifier {
        Identifier::parse_static(value)
    }

    #[test]
    fn mutable_access_is_limited_to_dynamic_entries() {
        static STATIC: [u32; 1] = [10];
        let dynamic = id("test:dynamic");
        let mut mapping = FxHashMap::default();
        mapping.insert(dynamic.clone(), 1);
        let mut registry = Registry::new(&STATIC, Box::new([20]), mapping);

        assert_eq!(registry.get_by_id_mut(0), None);
        *registry.get_mut(&dynamic).unwrap() = 25;

        assert_eq!(registry.get_by_id(0), Some(&10));
        assert_eq!(registry.get_by_id(1), Some(&25));
    }
}
