use std::sync::Arc;

use pumpkin_util::identifier::Identifier;

use crate::{Registry, RegistryAccess, error::RegistryGetError, key::DataKey};

#[derive(Clone)]
pub struct RegistryLookup(Arc<dyn RegistryAccess + Send + Sync>);

impl RegistryLookup {
    #[must_use]
    pub fn new(root: Arc<dyn RegistryAccess + Send + Sync>) -> Self {
        Self(root)
    }

    pub fn get<T>(&self, key: &DataKey<T>) -> Result<Arc<T>, RegistryGetError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let (item_id, registry_ids) = key.path().split_last().ok_or(RegistryGetError::EmptyPath)?;

        let registry = match registry_ids.split_last() {
            Some((registry_id, parent_ids)) => {
                let mut parent = self
                    .0
                    .clone()
                    .into_any()
                    .downcast::<RootRegistry>()
                    .map_err(|_| {
                        RegistryGetError::ExpectedRegistry(Identifier::from_static(
                            "lookup", "root",
                        ))
                    })?;

                for identifier in parent_ids {
                    let child = parent
                        .get(identifier)
                        .ok_or_else(|| RegistryGetError::NotFound(identifier.clone()))?;

                    parent = child
                        .into_any()
                        .downcast::<RootRegistry>()
                        .map_err(|_| RegistryGetError::ExpectedRegistry(identifier.clone()))?;
                }

                parent
                    .get(registry_id)
                    .ok_or_else(|| RegistryGetError::NotFound(registry_id.clone()))
            }
            None => Ok(self.0.clone()),
        }?;

        let expected = registry.type_name();

        let registry = registry.into_any().downcast::<Registry<T>>().map_err(|_| {
            RegistryGetError::TypeMismatch {
                identifier: item_id.clone(),
                expected,
            }
        })?;

        registry
            .get(item_id)
            .ok_or_else(|| RegistryGetError::NotFound(item_id.clone()))
    }
}

type RootRegistry = Registry<dyn RegistryAccess + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegistryAccess;
    use pumpkin_util::identifier::Identifier;

    fn id(value: &'static str) -> Identifier {
        Identifier::from_static("test", value)
    }

    fn register_registry<T>(
        parent: &RootRegistry,
        identifier: Identifier,
        registry: Arc<Registry<T>>,
    ) where
        T: ?Sized + Send + Sync + 'static,
    {
        let erased: Arc<dyn RegistryAccess + Send + Sync> = registry;
        parent.register_arc(identifier, erased).unwrap();
    }

    #[test]
    fn resolves_value_from_root_registry() {
        let root = Arc::new(RootRegistry::new());
        let numbers = Arc::new(Registry::new());
        numbers.register(id("answer"), 42u32).unwrap();
        register_registry(&root, id("numbers"), numbers);

        let key = DataKey::<u32>::builder(id("numbers"))
            .add_subkey(id("answer"))
            .build();

        assert_eq!(*RegistryLookup::new(root).get(&key).unwrap(), 42);
    }

    #[test]
    fn resolves_value_through_multiple_nested_registries() {
        let root = Arc::new(RootRegistry::new());
        let namespace = Arc::new(RootRegistry::new());
        let gameplay = Arc::new(RootRegistry::new());
        let names = Arc::new(Registry::new());
        names.register(id("player"), String::from("Steve")).unwrap();

        register_registry(&gameplay, id("names"), names);
        register_registry(&namespace, id("gameplay"), gameplay);
        register_registry(&root, id("namespace"), namespace);

        let key = DataKey::<String>::builder(id("namespace"))
            .add_subkey(id("gameplay"))
            .add_subkey(id("names"))
            .add_subkey(id("player"))
            .build();

        assert_eq!(&*RegistryLookup::new(root).get(&key).unwrap(), "Steve");
    }

    #[test]
    fn one_element_path_is_accepted() {
        let root = Arc::new(Registry::new());
        root.register(id("answer"), 42u32).unwrap();

        let key = DataKey::<u32>::builder(id("answer")).build();

        assert_eq!(*RegistryLookup::new(root).get(&key).unwrap(), 42);
    }

    #[test]
    fn missing_intermediate_registry_returns_not_found() {
        let root = Arc::new(RootRegistry::new());
        let key = DataKey::<u32>::builder(id("missing-parent"))
            .add_subkey(id("numbers"))
            .add_subkey(id("answer"))
            .build();

        assert!(matches!(
            RegistryLookup::new(root).get(&key),
            Err(RegistryGetError::NotFound(identifier)) if identifier == id("missing-parent")
        ));
    }

    #[test]
    fn non_registry_intermediate_entry_returns_expected_registry() {
        let root = Arc::new(RootRegistry::new());
        let values = Arc::new(Registry::new());
        values.register(id("entry"), 5u32).unwrap();
        register_registry(&root, id("not-a-root"), values);

        let key = DataKey::<u32>::builder(id("not-a-root"))
            .add_subkey(id("numbers"))
            .add_subkey(id("answer"))
            .build();

        assert!(matches!(
            RegistryLookup::new(root).get(&key),
            Err(RegistryGetError::ExpectedRegistry(identifier)) if identifier == id("not-a-root")
        ));
    }

    #[test]
    fn wrong_registry_value_type_returns_type_mismatch() {
        let root = Arc::new(RootRegistry::new());
        let strings = Arc::new(Registry::new());
        strings
            .register(id("answer"), String::from("forty-two"))
            .unwrap();
        register_registry(&root, id("values"), strings);

        let key = DataKey::<u32>::builder(id("values"))
            .add_subkey(id("answer"))
            .build();

        assert!(matches!(
            RegistryLookup::new(root).get(&key),
            Err(RegistryGetError::TypeMismatch { identifier, expected }) if identifier == id("answer") && expected == "alloc::string::String"
        ));
    }

    #[test]
    fn missing_item_returns_not_found() {
        let root = Arc::new(RootRegistry::new());
        register_registry(&root, id("numbers"), Arc::new(Registry::<u32>::new()));

        let key = DataKey::<u32>::builder(id("numbers"))
            .add_subkey(id("missing"))
            .build();

        assert!(matches!(
            RegistryLookup::new(root).get(&key),
            Err(RegistryGetError::NotFound(identifier)) if identifier == id("missing")
        ));
    }

    #[test]
    fn cloned_lookup_observes_later_registrations() {
        let root = Arc::new(RootRegistry::new());
        let numbers = Arc::new(Registry::new());
        register_registry(&root, id("numbers"), Arc::clone(&numbers));
        let lookup = RegistryLookup::new(root);
        let cloned = lookup.clone();

        numbers.register(id("answer"), 42u32).unwrap();
        let key = DataKey::<u32>::builder(id("numbers"))
            .add_subkey(id("answer"))
            .build();

        assert_eq!(*lookup.get(&key).unwrap(), 42);
        assert_eq!(*cloned.get(&key).unwrap(), 42);
    }
}
