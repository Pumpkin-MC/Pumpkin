#![allow(clippy::unwrap_used)]

use std::any::type_name;
use std::sync::Arc;

use pumpkin_registry::{
    DataKey, Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider,
    error::DataKeyGetError,
};
use pumpkin_util::identifier::Identifier;

const fn id(value: &'static str) -> Identifier {
    Identifier::parse_static(value)
}

bootstrap_provider! {
    NUMBERS: u32 => "test:numbers_registry" => {
        "test:one" => 10,
        "test:two" => 20,
    }
}

bootstrap_provider! {
    DEEP_NUMBERS: u32 => "test:deep_numbers_registry" => {
        "test:value" => 99,
    }
}

bootstrap_provider! {
    ROOT: Arc<dyn Registry> => "test:root_registry",
    || {
        vec![RegistryEntry::new(
            id("test:numbers"),
            RegistryBuilder::<u32>::frozen(&id("test:numbers_registry"))
                .unwrap()
                .arc_dyn(),
        )]
    }
}

bootstrap_provider! {
    BRANCH: Arc<dyn Registry> => "test:branch_registry",
    || {
        vec![RegistryEntry::new(
            id("test:numbers"),
            RegistryBuilder::<u32>::frozen(&id("test:deep_numbers_registry"))
                .unwrap()
                .arc_dyn(),
        )]
    }
}

bootstrap_provider! {
    DEEP_ROOT: Arc<dyn Registry> => "test:deep_root_registry",
    || {
        vec![RegistryEntry::new(
            id("test:branch"),
            RegistryBuilder::<Arc<dyn Registry>>::frozen(
                &id("test:branch_registry"),
            )
            .unwrap()
            .arc_dyn(),
        )]
    }
}

bootstrap_provider! {
    NON_REGISTRY_ROOT: u32 => "test:non_registry_root" => {
        "test:not_a_registry" => 123,
    }
}

fn nested_root() -> Arc<dyn Registry> {
    RegistryBuilder::<Arc<dyn Registry>>::frozen(&id("test:root_registry"))
        .unwrap()
        .arc_dyn()
}

fn deeply_nested_root() -> Arc<dyn Registry> {
    RegistryBuilder::<Arc<dyn Registry>>::frozen(&id("test:deep_root_registry"))
        .unwrap()
        .arc_dyn()
}

#[tokio::test]
async fn data_key_resolves_value() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:two");

    assert_eq!(*key.get(root.as_ref()).await.unwrap(), 20);
}

#[tokio::test]
async fn data_key_can_be_reused_after_initial_resolution() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:two");

    assert_eq!(*key.get(root.as_ref()).await.unwrap(), 20);
    assert_eq!(*key.get(root.as_ref()).await.unwrap(), 20);
}

#[tokio::test]
async fn data_key_walks_multiple_nested_registry_levels() {
    let root = deeply_nested_root();
    let key = DataKey::<u32>::new("test:branch/test:numbers/test:value");

    assert_eq!(*key.get(root.as_ref()).await.unwrap(), 99);
}

#[tokio::test]
async fn get_reports_missing_registry_identifier() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:missing/test:value");

    let result = key.get(root.as_ref()).await;

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingIdentifier { identifier })
            if identifier == id("test:missing")
    ));
}

#[tokio::test]
async fn get_reports_non_registry_path_entry() {
    let root = RegistryBuilder::<u32>::frozen(&id("test:non_registry_root"))
        .unwrap()
        .arc_dyn();

    let key = DataKey::<u32>::new("test:not_a_registry/test:value");

    let result = key.get(root.as_ref()).await;

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingRegistry { id }) if id == 0
    ));
}

#[tokio::test]
async fn get_reports_missing_value_identifier() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:missing_value");

    let result = key.get(root.as_ref()).await;

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingIdentifier { identifier })
            if identifier == id("test:missing_value")
    ));
}

#[tokio::test]
async fn get_reports_value_type_mismatch() {
    let root = nested_root();
    let key = DataKey::<u64>::new("test:numbers/test:one");

    assert!(matches!(
        key.get(root.as_ref()).await,
        Err(DataKeyGetError::TypeMismatch { expected, actual })
            if expected == type_name::<u64>()
                && actual == type_name::<u32>()
    ));
}

#[test]
fn blocking_data_key_resolves_value_without_runtime() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:two");

    assert_eq!(*key.get_blocking(root.as_ref()).unwrap(), 20);
}

#[test]
fn blocking_data_key_can_be_reused_after_initial_resolution() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:one");

    assert_eq!(*key.get_blocking(root.as_ref()).unwrap(), 10);
    assert_eq!(*key.get_blocking(root.as_ref()).unwrap(), 10);
}

#[test]
fn blocking_get_reports_missing_registry_identifier() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:missing_registry/test:value");

    let result = key.get_blocking(root.as_ref());

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingIdentifier { identifier })
            if identifier == id("test:missing_registry")
    ));
}

#[test]
fn blocking_get_reports_missing_value_identifier() {
    let root = nested_root();
    let key = DataKey::<u32>::new("test:numbers/test:missing_value");

    let result = key.get_blocking(root.as_ref());

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingIdentifier { identifier })
            if identifier == id("test:missing_value")
    ));
}
