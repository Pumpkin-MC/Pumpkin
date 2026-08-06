use crate::error::{RegistryInitError, RegistryInsertError};
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

pub struct RegistryBuilder<T: Send + Sync + 'static> {
    pub(crate) static_entries: &'static [T],
    pub(crate) entries: Vec<T>,
    pub(crate) mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> RegistryBuilder<T> {
    pub fn new(
        static_entries: &'static [T],
        static_identifiers: &[Identifier],
    ) -> Result<Self, RegistryInitError> {
        if static_entries.len() != static_identifiers.len() {
            return Err(RegistryInitError::MappingMismatch {
                values: static_entries.len(),
                identifiers: static_identifiers.len(),
            });
        }

        let mut builder = Self {
            static_entries,
            entries: Vec::new(),
            mapping: FxHashMap::default(),
        };

        for (index, item) in static_identifiers.iter().enumerate() {
            let None = builder.mapping.insert(item.clone(), index) else {
                return Err(RegistryInitError::AlreadyRegistered(item.clone()));
            };
        }

        Ok(builder)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Identifier {
        Identifier::parse_static(value)
    }

    #[test]
    fn initialization_rejects_mapping_length_mismatch() {
        static VALUES: [u32; 1] = [10];

        let Err(error) = RegistryBuilder::new(&VALUES, &[]) else {
            panic!("expected mapping mismatch");
        };

        assert!(matches!(
            error,
            RegistryInitError::MappingMismatch {
                values: 1,
                identifiers: 0
            }
        ));
    }

    #[test]
    fn initialization_rejects_duplicate_static_identifiers() {
        static VALUES: [u32; 2] = [10, 20];
        let duplicate = id("test:duplicate");

        let Err(error) = RegistryBuilder::new(&VALUES, &[duplicate.clone(), duplicate.clone()])
        else {
            panic!("expected duplicate static identifier");
        };

        assert!(matches!(
            error,
            RegistryInitError::AlreadyRegistered(found) if found == duplicate
        ));
    }

    #[test]
    fn static_and_dynamic_entries_have_stable_contiguous_ids() {
        static VALUES: [u32; 2] = [10, 20];
        let static_ids = [id("test:static_a"), id("test:static_b")];
        let first = id("test:first");
        let second = id("test:second");
        let mut builder = RegistryBuilder::new(&VALUES, &static_ids).unwrap();

        builder.register(first.clone(), 30).unwrap();
        builder.register(second.clone(), 40).unwrap();

        assert_eq!(builder.len(), 4);
        assert!(!builder.is_empty());
        assert_eq!(builder.get_id(&static_ids[0]), Some(0));
        assert_eq!(builder.get_id(&static_ids[1]), Some(1));
        assert_eq!(builder.get_id(&first), Some(2));
        assert_eq!(builder.get_id(&second), Some(3));
        assert_eq!(builder.get_by_id(0), Some(&10));
        assert_eq!(builder.get_by_id(1), Some(&20));
        assert_eq!(builder.get_by_id(2), Some(&30));
        assert_eq!(builder.get_by_id(3), Some(&40));
        assert_eq!(builder.get_by_id(4), None);
    }

    #[test]
    fn duplicate_registration_is_atomic() {
        let identifier = id("test:value");
        let mut builder = RegistryBuilder::new(&[], &[]).unwrap();
        builder.register(identifier.clone(), 1u32).unwrap();

        let error = builder.register(identifier.clone(), 2u32).unwrap_err();

        assert!(matches!(
            error,
            RegistryInsertError::AlreadyRegistered(found) if found == identifier
        ));
        assert_eq!(builder.len(), 1);
        assert_eq!(builder.get(&identifier), Some(&1));
        assert_eq!(builder.get_by_id(1), None);
    }

    #[test]
    fn iteration_returns_every_identifier_value_pair() {
        let first = id("test:first");
        let second = id("test:second");
        let mut builder = RegistryBuilder::new(&[], &[]).unwrap();
        builder.register(first.clone(), 11u32).unwrap();
        builder.register(second.clone(), 22u32).unwrap();

        let mut values = builder
            .iter()
            .map(|(identifier, value)| (identifier.clone(), *value))
            .collect::<Vec<_>>();
        values.sort_by_key(|(_, value)| *value);

        assert_eq!(values, vec![(first, 11), (second, 22)]);
    }
}
