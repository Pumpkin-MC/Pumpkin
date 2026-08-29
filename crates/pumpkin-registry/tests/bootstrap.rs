#![allow(clippy::unwrap_used)]

use pumpkin_registry::{
    BOOTSTRAP, Registry, RegistryBuilder, RegistryConfig, TypedRegistry,
    bootstrap::{BootstrapManager, RegistryEntry},
    bootstrap_provider,
};
use pumpkin_util::identifier::Identifier;

#[derive(Debug, PartialEq, Eq)]
struct Block(usize);

static BLOCK_REGISTRY: Identifier = Identifier::parse_static("test:blocks");
static OVERWRITE_REGISTRY: Identifier = Identifier::parse_static("test:overwrite");

bootstrap_provider! {
    PROVIDER_ONE: Block => "test:blocks" => {
        "test:one" => Block(1),
        "test:two" => Block(2),
    }
}

bootstrap_provider! {
    PROVIDER_TWO: Block => "test:blocks" => {
        "test:three" => Block(3),
    }
}

bootstrap_provider! {
    PROVIDER_THREE: Block => "test:blocks",
    || {
        vec![RegistryEntry::new(Identifier::parse_static("test:four"), Block(4))]
    }
}

bootstrap_provider! {
    OVERWRITE_PROVIDER_ONE: Block => "test:overwrite" => {
        "test:value" => Block(1),
    }
}

bootstrap_provider! {
    OVERWRITE_PROVIDER_TWO: Block => "test:overwrite" => {
        "test:value" => Block(2),
    }
}

#[test]
fn discovers_linker_section_providers() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    assert_eq!(
        BOOTSTRAP
            .get()
            .unwrap()
            .providers_for(&BLOCK_REGISTRY)
            .count(),
        3,
    );
}

#[test]
fn populates_all_linker_section_providers() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());

    let (entries, mapping) = BOOTSTRAP
        .get()
        .unwrap()
        .populate::<Block>(&BLOCK_REGISTRY)
        .unwrap();

    assert_eq!(entries.len(), 4);

    let id = mapping.get(&Identifier::parse_static("test:one"));
    assert!(id.is_some());
    assert_eq!(entries[*id.unwrap()], Block(1));

    let id = mapping.get(&Identifier::parse_static("test:two"));
    assert!(id.is_some());
    assert_eq!(entries[*id.unwrap()], Block(2));

    let id = mapping.get(&Identifier::parse_static("test:three"));
    assert!(id.is_some());
    assert_eq!(entries[*id.unwrap()], Block(3));

    let id = mapping.get(&Identifier::parse_static("test:four"));
    assert!(id.is_some());
    assert_eq!(entries[*id.unwrap()], Block(4));
}

#[test]
fn duplicate_entries_are_rejected_by_default() {
    let manager = BootstrapManager::new();

    assert!(manager.populate::<Block>(&OVERWRITE_REGISTRY).is_err());
}

#[test]
fn duplicate_entries_can_be_overwritten() {
    let manager = BootstrapManager::new();
    let config = RegistryConfig {
        allow_overwrites: true,
    };

    let (entries, mapping) = manager
        .populate_with_config::<Block>(&OVERWRITE_REGISTRY, config)
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(mapping.len(), 1);
    assert_eq!(mapping[&Identifier::parse_static("test:value")], 0);
    assert!(matches!(entries[0], Block(1 | 2)));
}

#[test]
fn replacement_preserves_bootstrap_entries() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<Block>::reloadable(&BLOCK_REGISTRY, &[], &[]).unwrap();

    registry
        .replace_entries([
            (Identifier::parse_static("test:new_one"), Block(10)),
            (Identifier::parse_static("test:new_two"), Block(20)),
        ])
        .unwrap();

    let new_one = Identifier::parse_static("test:new_one");
    let old_one = Identifier::parse_static("test:one");
    assert_eq!(*registry.get(&new_one).unwrap(), Block(10));
    assert_eq!(*registry.get(&old_one).unwrap(), Block(1));
}

#[test]
fn replacement_rejects_bootstrap_overwrite_by_default() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<Block>::reloadable(&BLOCK_REGISTRY, &[], &[]).unwrap();
    let original = Identifier::parse_static("test:one");

    let result = registry.replace_entries([(original.clone(), Block(10))]);

    assert!(result.is_err());
    assert_eq!(*registry.get(&original).unwrap(), Block(1));
}

#[test]
fn replacement_overwrites_bootstrap_entry_when_configured() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<Block>::reloadable_with_config(
        &BLOCK_REGISTRY,
        &[],
        &[],
        RegistryConfig {
            allow_overwrites: true,
        },
    )
    .unwrap();
    let original = Identifier::parse_static("test:one");
    let untouched = Identifier::parse_static("test:two");
    let original_id = registry.get_id(&original).unwrap();

    registry
        .replace_entries([(original.clone(), Block(10))])
        .unwrap();

    assert_eq!(registry.get_id(&original), Some(original_id));
    assert_eq!(*registry.get(&original).unwrap(), Block(10));
    assert_eq!(*registry.get(&untouched).unwrap(), Block(2));
}

#[test]
fn failed_replacement_preserves_existing_snapshot() {
    let _ = BOOTSTRAP.set(BootstrapManager::new());
    let registry = RegistryBuilder::<Block>::reloadable(&BLOCK_REGISTRY, &[], &[]).unwrap();
    let retained = Identifier::parse_static("test:retained");

    registry
        .replace_entries([(retained.clone(), Block(10))])
        .unwrap();

    let duplicate = Identifier::parse_static("test:duplicate");
    let result = registry.replace_entries([(duplicate.clone(), Block(20)), (duplicate, Block(30))]);

    assert!(result.is_err());
    assert_eq!(*registry.get(&retained).unwrap(), Block(10));
}
