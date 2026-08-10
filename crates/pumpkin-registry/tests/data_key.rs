#![allow(clippy::unwrap_used)]

use std::any::type_name;
use std::sync::Arc;

use pumpkin_registry::{DataKey, Registry, RegistryBuilder, error::DataKeyGetError};
use pumpkin_util::identifier::Identifier;

const fn id(value: &'static str) -> Identifier {
    Identifier::parse_static(value)
}

fn frozen_registry<T>(
    name: &'static str,
    entries: Vec<T>,
    identifiers: &[Identifier],
) -> Arc<dyn Registry>
where
    T: Send + Sync + 'static,
{
    RegistryBuilder::frozen(&id(name), entries, identifiers)
        .unwrap()
        .arc_dyn()
}

fn nested_root() -> Arc<dyn Registry> {
    let numbers = frozen_registry(
        "test:numbers_registry",
        vec![10u32, 20u32],
        &[id("test:one"), id("test:two")],
    );

    frozen_registry("test:root_registry", vec![numbers], &[id("test:numbers")])
}

fn deeply_nested_root() -> Arc<dyn Registry> {
    let numbers = frozen_registry(
        "test:deep_numbers_registry",
        vec![99u32],
        &[id("test:value")],
    );

    let branch = frozen_registry("test:branch_registry", vec![numbers], &[id("test:numbers")]);

    frozen_registry(
        "test:deep_root_registry",
        vec![branch],
        &[id("test:branch")],
    )
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

    // First lookup resolves and caches the numeric path.
    assert_eq!(*key.get(root.as_ref()).await.unwrap(), 20);

    // Subsequent lookup uses the cached numeric path.
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
    let root = frozen_registry(
        "test:non_registry_root",
        vec![123u32],
        &[id("test:not_a_registry")],
    );

    let key = DataKey::<u32>::new("test:not_a_registry/test:value");

    let result = key.get(root.as_ref()).await;

    assert!(matches!(
        result,
        Err(DataKeyGetError::MissingRegistry { id })
            if id == 0
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
