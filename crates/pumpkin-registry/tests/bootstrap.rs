#![allow(clippy::unwrap_used)]

use pumpkin_registry::{
    BOOTSTRAP,
    bootstrap::{BootstrapManager, RegistryEntry},
    bootstrap_provider,
};
use pumpkin_util::identifier::Identifier;

#[derive(Debug, PartialEq, Eq)]
struct Block(usize);

static BLOCK_REGISTRY: Identifier = Identifier::parse_static("test:blocks");

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
