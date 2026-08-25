#![allow(clippy::unwrap_used)]

use pumpkin_registry::{
    BOOTSTRAP, RegistryBuilder, TypedRegistry, bootstrap::BootstrapManager, bootstrap_provider,
};
use pumpkin_util::identifier::Identifier;

bootstrap_provider! {
    RELOADABLE_VALUES: u32 => "test:reloadable_values" => {
        "test:one" => 10,
        "test:two" => 20,
    }
}

#[test]
fn reload_keeps_existing_snapshot_refs_alive() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<u32>::reloadable(
        &Identifier::parse_static("test:reloadable_values"),
        &[],
        &[],
    )
    .unwrap();

    let before = registry.get(&Identifier::parse_static("test:one")).unwrap();

    registry.reload().unwrap();

    assert_eq!(*before, 10);
    assert_eq!(
        *registry.get(&Identifier::parse_static("test:one")).unwrap(),
        10
    );
}

#[test]
fn reloadable_iteration_uses_a_stable_snapshot() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<u32>::reloadable(
        &Identifier::parse_static("test:reloadable_values"),
        &[],
        &[],
    )
    .unwrap();

    let mut iter = registry.iter();
    let first = iter.next().unwrap();

    registry.reload().unwrap();

    assert!(matches!(*first.1, 10 | 20));
    assert!(iter.all(|(_, value)| matches!(*value, 10 | 20)));
}
