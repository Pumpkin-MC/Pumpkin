use crate::{
    builder::RegistryBuilder,
    error::{RegistryInsertError, RegistryLockError},
    registry::Registry,
};
use pumpkin_util::identifier::Identifier;

pub enum RegistryHolder<T: Send + Sync + 'static> {
    Mutable(RegistryBuilder<T>),
    Immutable(Registry<T>),
    Mutating,
}

impl<T: Send + Sync + 'static> RegistryHolder<T> {
    pub fn new(static_entries: &'static [T]) -> Self {
        Self::Mutable(RegistryBuilder::new(static_entries))
    }

    pub fn lock(&mut self) -> Result<(), RegistryLockError> {
        match self {
            Self::Immutable(_) => return Ok(()),
            Self::Mutating => {
                return Err(RegistryLockError::Interrupted);
            }
            Self::Mutable(_) => {}
        }

        let Self::Mutable(builder) = std::mem::replace(self, Self::Mutating) else {
            panic!("the registry state was checked while holding the write lock");
        };

        *self = Self::Immutable(builder.build());
        Ok(())
    }

    #[must_use]
    pub const fn is_locked(&self) -> bool {
        matches!(self, Self::Immutable(_))
    }

    pub fn register(
        &mut self,
        identifier: Identifier,
        value: T,
    ) -> Result<(), RegistryInsertError> {
        match self {
            Self::Mutable(registry) => registry.register(identifier, value),
            _ => Err(RegistryInsertError::Immutable),
        }
    }

    #[must_use]
    pub fn get(&self, identifier: &Identifier) -> Option<&T> {
        match self {
            Self::Mutable(registry) => registry.get(identifier),
            Self::Immutable(registry) => registry.get(identifier),
            Self::Mutating => None,
        }
    }

    #[must_use]
    pub fn get_by_id(&self, id: usize) -> Option<&T> {
        match self {
            Self::Mutable(registry) => registry.get_by_id(id),
            Self::Immutable(registry) => registry.get_by_id(id),
            Self::Mutating => None,
        }
    }

    #[must_use]
    pub fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        match self {
            Self::Mutable(registry) => registry.get_id(identifier),
            Self::Immutable(registry) => registry.get_id(identifier),
            Self::Mutating => None,
        }
    }

    #[must_use]
    pub fn contains(&self, identifier: &Identifier) -> bool {
        match self {
            Self::Mutable(registry) => registry.contains(identifier),
            Self::Immutable(registry) => registry.contains(identifier),
            Self::Mutating => false,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Mutable(registry) => registry.len(),
            Self::Immutable(registry) => registry.len(),
            Self::Mutating => 0,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Mutable(registry) => registry.is_empty(),
            Self::Immutable(registry) => registry.is_empty(),
            Self::Mutating => true,
        }
    }

    #[allow(clippy::iter_on_empty_collections)]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        match self {
            Self::Mutable(registry) => registry.iter(),
            Self::Immutable(registry) => registry.iter(),
            Self::Mutating => [].iter().chain([].iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Identifier {
        Identifier::parse_static(value)
    }

    #[test]
    fn lock_is_idempotent_and_preserves_entries() {
        let mut holder = RegistryHolder::new(&[]);
        let identifier = id("test:value");
        holder.register(identifier.clone(), 42u32).unwrap();

        holder.lock().unwrap();
        holder.lock().unwrap();

        assert!(holder.is_locked());
        assert_eq!(holder.get(&identifier), Some(&42));
        assert_eq!(holder.get_by_id(0), Some(&42));
        assert_eq!(holder.get_id(&identifier), Some(0));
    }

    #[test]
    fn registration_is_rejected_after_locking() {
        let mut holder = RegistryHolder::new(&[]);
        holder.lock().unwrap();

        assert!(matches!(
            holder.register(id("test:value"), 1u32),
            Err(RegistryInsertError::Immutable)
        ));
    }

    #[test]
    fn mutating_state_reports_interruption_and_has_no_visible_values() {
        let mut holder = RegistryHolder::<u32>::Mutating;

        assert!(matches!(holder.lock(), Err(RegistryLockError::Interrupted)));
        assert!(!holder.is_locked());
        assert!(holder.is_empty());
        assert_eq!(holder.len(), 0);
        assert_eq!(holder.get(&id("test:value")), None);
        assert_eq!(holder.get_by_id(0), None);
        assert_eq!(holder.iter().count(), 0);
    }
}
