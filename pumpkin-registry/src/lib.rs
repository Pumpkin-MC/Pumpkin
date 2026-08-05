use crate::error::RegistryLockError;
use pumpkin_util::identifier::Identifier;
use std::any::{Any, TypeId, type_name};

mod builder;
mod holder;
mod key;
mod registry;

pub mod error;
pub use crate::holder::RegistryHolder;
pub use crate::key::{ArcDataKey, DataKeyBuilder, RefDataKey};

pub trait LockableRegistry: Any + Send + Sync {
    fn lock(&mut self) -> Result<(), RegistryLockError>;
    fn is_locked(&self) -> bool;

    fn type_id(&self) -> TypeId;
    fn type_name(&self) -> &'static str;

    fn get_id(&self, identifier: &Identifier) -> Option<usize>;
    fn get_by_id(&self, id: usize) -> Option<&(dyn Any + Send + Sync)>;
}

pub type ErasedRegistry = Box<dyn LockableRegistry>;
pub type NestRegistry = RegistryHolder<ErasedRegistry>;

impl<T: Send + Sync + 'static> LockableRegistry for RegistryHolder<T> {
    fn lock(&mut self) -> Result<(), RegistryLockError> {
        Self::lock(self)
    }

    fn is_locked(&self) -> bool {
        Self::is_locked(self)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.get_id(identifier)
    }

    fn get_by_id(&self, id: usize) -> Option<&(dyn Any + Send + Sync)> {
        self.get_by_id(id)
            .map(|value| value as &(dyn Any + Send + Sync))
    }
}

impl NestRegistry {
    pub fn lock_recursive(&mut self) -> Result<(), RegistryLockError> {
        match self {
            Self::Immutable(_) => return Ok(()),
            Self::Mutating => return Err(RegistryLockError::Interrupted),
            Self::Mutable(builder) => {
                for registry in builder.iter_mut() {
                    registry.lock()?;
                }
            }
        }

        self.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DataKeyBuildError, RegistryInsertError};
    use std::sync::Arc;

    fn id(value: &'static str) -> Identifier {
        Identifier::parse_static(value)
    }

    fn number_registry() -> RegistryHolder<u32> {
        let mut registry = RegistryHolder::new(&[]);
        registry.register(id("test:one"), 1).unwrap();
        registry.register(id("test:two"), 2).unwrap();
        registry
    }

    fn nested_root() -> NestRegistry {
        let mut root = NestRegistry::new(&[]);
        root.register(id("test:numbers"), Box::new(number_registry()))
            .unwrap();
        root
    }

    #[test]
    fn recursive_lock_freezes_root_and_children() {
        let mut root = nested_root();

        root.lock_recursive().unwrap();

        assert!(root.is_locked());
        let child = root.get(&id("test:numbers")).unwrap();
        assert!(child.is_locked());
    }

    #[test]
    fn recursive_lock_is_idempotent() {
        let mut root = nested_root();

        root.lock_recursive().unwrap();
        root.lock_recursive().unwrap();

        assert!(root.is_locked());
    }

    #[test]
    fn recursive_lock_stops_on_interrupted_child_without_freezing_root() {
        let mut root = NestRegistry::new(&[]);
        root.register(
            id("test:interrupted"),
            Box::new(RegistryHolder::<u32>::Mutating),
        )
        .unwrap();

        assert!(matches!(
            root.lock_recursive(),
            Err(RegistryLockError::Interrupted)
        ));
        assert!(!root.is_locked());
        assert!(matches!(
            root.register(id("test:other"), Box::new(number_registry())),
            Ok(())
        ));
    }

    #[test]
    fn ref_data_key_resolves_nested_value_and_exposes_numeric_path() {
        let mut root = nested_root();
        root.lock_recursive().unwrap();

        let key = DataKeyBuilder::<u32>::new()
            .child(id("test:numbers"))
            .child(id("test:two"))
            .build_ref(&root)
            .unwrap();

        assert_eq!(key.ids(), &[0, 1]);
        assert_eq!(key.get().unwrap(), &2);
        assert_eq!(key.get().unwrap(), &2);
    }

    #[test]
    fn arc_data_key_keeps_registry_alive() {
        let mut root = Arc::new(nested_root());
        Arc::get_mut(&mut root).unwrap().lock_recursive().unwrap();

        let key = DataKeyBuilder::<u32>::new()
            .child(id("test:numbers"))
            .child(id("test:one"))
            .build_arc(&root)
            .unwrap();
        drop(root);

        assert_eq!(key.ids(), &[0, 0]);
        assert_eq!(key.get().unwrap(), &1);
        assert_eq!(key.get().unwrap(), &1);
    }

    #[test]
    fn data_key_builder_rejects_empty_path() {
        let root = nested_root();

        assert!(matches!(
            DataKeyBuilder::<u32>::new().build_ref(&root),
            Err(DataKeyBuildError::Empty)
        ));
    }

    #[test]
    fn data_key_builder_reports_missing_registry() {
        let root = nested_root();
        let missing = id("test:missing");

        let Err(error) = DataKeyBuilder::<u32>::new()
            .child(missing.clone())
            .child(id("test:value"))
            .build_ref(&root)
        else {
            panic!("expected missing registry error")
        };

        assert!(matches!(
            error,
            DataKeyBuildError::MissingRegistry(found) if found == missing
        ));
    }

    #[test]
    fn data_key_builder_reports_non_registry_path_component() {
        let root = nested_root();
        let value = id("test:one");

        let Err(error) = DataKeyBuilder::<u32>::new()
            .child(id("test:numbers"))
            .child(value.clone())
            .child(id("test:deeper"))
            .build_ref(&root)
        else {
            panic!("expected non-registry path error")
        };

        assert!(matches!(
            error,
            DataKeyBuildError::NotARegistry(found) if found == value
        ));
    }

    #[test]
    fn data_key_builder_reports_registry_value_type_mismatch() {
        let root = nested_root();

        let Err(error) = DataKeyBuilder::<u32>::new()
            .child(id("test:numbers"))
            .child(id("test:one"))
            .build_ref(&root)
        else {
            panic!("expected registry type mismatch")
        };

        assert!(matches!(error, DataKeyBuildError::TypeMismatch { .. }));
    }

    #[test]
    fn data_key_builder_reports_missing_value() {
        let root = nested_root();
        let missing = id("test:missing");

        let Err(error) = DataKeyBuilder::<u32>::new()
            .child(id("test:numbers"))
            .child(missing.clone())
            .build_ref(&root)
        else {
            panic!("expected missing value error")
        };

        assert!(matches!(
            error,
            DataKeyBuildError::MissingValue(found) if found == missing
        ));
    }

    #[test]
    fn locking_prevents_new_nested_registries() {
        let mut root = nested_root();
        root.lock_recursive().unwrap();

        assert!(matches!(
            root.register(id("test:other"), Box::new(number_registry())),
            Err(RegistryInsertError::Immutable)
        ));
    }
}
