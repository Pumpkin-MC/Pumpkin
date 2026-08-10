use std::any::type_name;
use std::sync::Arc;

use pumpkin_registry::{
    DataKey, DataKeyBuilder, Registry, RegistryBuilder,
    error::{DataKeyBuildError, DataKeyGetError},
};
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
async fn ref_data_key_builds_numeric_path_and_resolves_value() {
    let root = nested_root();
    let key = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(id("test:two"))
        .build_ref(root.as_ref())
        .await
        .unwrap();

    assert_eq!(key.ids(), &[0, 1]);
    assert_eq!(*key.get().await.unwrap(), 20);
}

#[tokio::test]
async fn arc_data_key_keeps_registry_tree_alive() {
    let root = nested_root();
    let key = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(id("test:one"))
        .build_arc(&root)
        .await
        .unwrap();

    drop(root);

    assert_eq!(key.ids(), &[0, 0]);
    assert_eq!(*key.get().await.unwrap(), 10);
}

#[tokio::test]
async fn data_key_walks_multiple_nested_registry_levels() {
    let root = deeply_nested_root();
    let key = DataKeyBuilder::<u32>::new(id("test:branch"))
        .child(id("test:numbers"))
        .child(id("test:value"))
        .build_ref(root.as_ref())
        .await
        .unwrap();

    assert_eq!(key.ids(), &[0, 0, 0]);
    assert_eq!(*key.get().await.unwrap(), 99);
}

#[tokio::test]
async fn builder_reports_missing_registry() {
    let root = nested_root();
    let missing = id("test:missing");

    let result = DataKeyBuilder::<u32>::new(missing.clone())
        .child(id("test:value"))
        .build_ref(root.as_ref())
        .await;

    assert!(matches!(
        result,
        Err(DataKeyBuildError::MissingRegistry(identifier)) if identifier == missing
    ));
}

#[tokio::test]
async fn builder_reports_non_registry_path_entry() {
    let root = frozen_registry(
        "test:non_registry_root",
        vec![123u32],
        &[id("test:not_a_registry")],
    );
    let target = id("test:not_a_registry");

    let result = DataKeyBuilder::<u32>::new(target.clone())
        .child(id("test:value"))
        .build_ref(root.as_ref())
        .await;

    assert!(matches!(
        result,
        Err(DataKeyBuildError::NotARegistry(identifier)) if identifier == target
    ));
}

#[tokio::test]
async fn builder_reports_missing_value() {
    let root = nested_root();
    let missing = id("test:missing_value");

    let result = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(missing.clone())
        .build_ref(root.as_ref())
        .await;

    assert!(matches!(
        result,
        Err(DataKeyBuildError::MissingValue(identifier)) if identifier == missing
    ));
}

#[tokio::test]
async fn get_reports_value_type_mismatch() {
    let root = nested_root();
    let key = DataKeyBuilder::<u64>::new(id("test:numbers"))
        .child(id("test:one"))
        .build_ref(root.as_ref())
        .await
        .unwrap();

    assert!(matches!(
        key.get().await,
        Err(DataKeyGetError::TypeMismatch { expected, actual })
            if expected == type_name::<u64>() && actual == type_name::<u32>()
    ));
}

#[test]
fn blocking_ref_data_key_build_and_get_work_without_runtime() {
    let root = nested_root();
    let key = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(id("test:two"))
        .build_ref_blocking(root.as_ref())
        .unwrap();

    assert_eq!(key.ids(), &[0, 1]);
    assert_eq!(*key.get_blocking().unwrap(), 20);
}

#[test]
fn blocking_arc_data_key_keeps_tree_alive() {
    let root = nested_root();
    let key = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(id("test:one"))
        .build_arc_blocking(&root)
        .unwrap();

    drop(root);

    assert_eq!(*key.get_blocking().unwrap(), 10);
}

#[test]
fn blocking_builder_reports_structural_errors() {
    let root = nested_root();

    let missing_registry = id("test:missing_registry");
    let result = DataKeyBuilder::<u32>::new(missing_registry.clone())
        .child(id("test:value"))
        .build_ref_blocking(root.as_ref());
    assert!(matches!(
        result,
        Err(DataKeyBuildError::MissingRegistry(identifier)) if identifier == missing_registry
    ));

    let missing_value = id("test:missing_value");
    let result = DataKeyBuilder::<u32>::new(id("test:numbers"))
        .child(missing_value.clone())
        .build_ref_blocking(root.as_ref());
    assert!(matches!(
        result,
        Err(DataKeyBuildError::MissingValue(identifier)) if identifier == missing_value
    ));
}
