#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use pumpkin_registry::{
    BOOTSTRAP, Registry, RegistryBuilder, RegistryResolvable, RegistryResolvableSet,
    bootstrap::BootstrapManager, bootstrap_provider, error::DataKeyGetError,
};
use pumpkin_util::identifier::Identifier;

const fn id(value: &'static str) -> Identifier {
    Identifier::parse_static(value)
}

bootstrap_provider! {
    NUMBERS: u32 => "test:resolvable_numbers" => {
        "test:one" => 10,
        "test:two" => 20,
    }
}

fn numbers() -> Arc<dyn Registry> {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    RegistryBuilder::<u32>::frozen(&id("test:resolvable_numbers"))
        .unwrap()
        .arc_dyn()
}

#[test]
fn registry_resolvable_resolves_against_selected_registry() {
    let registry = numbers();
    let resolvable = RegistryResolvable::<u32>::new(id("test:two"));

    assert_eq!(*resolvable.resolve(registry.as_ref()).unwrap(), 20);
}

#[tokio::test]
async fn registry_resolvable_resolves_async() {
    let registry = numbers();
    let resolvable = RegistryResolvable::<u32>::new(id("test:one"));

    assert_eq!(
        *resolvable.resolve_async(registry.as_ref()).await.unwrap(),
        10
    );
}

#[test]
fn registry_resolvable_reports_missing_identifier() {
    let registry = numbers();
    let resolvable = RegistryResolvable::<u32>::new(id("test:missing"));

    assert!(matches!(
        resolvable.resolve(registry.as_ref()),
        Err(DataKeyGetError::MissingIdentifier { identifier })
            if identifier == id("test:missing")
    ));
}

#[test]
fn registry_resolvable_set_preserves_shape() {
    let single = RegistryResolvableSet::single(RegistryResolvable::<u32>::new(id("test:one")));
    assert_eq!(single.as_single().unwrap().identifier(), &id("test:one"));

    let tag = RegistryResolvableSet::<u32>::tag(id("test:numbers"));
    assert_eq!(tag.as_tag(), Some(&id("test:numbers")));

    let list = RegistryResolvableSet::list([
        RegistryResolvable::<u32>::new(id("test:one")),
        RegistryResolvable::<u32>::new(id("test:two")),
    ]);
    assert_eq!(list.as_list().unwrap().len(), 2);
}
