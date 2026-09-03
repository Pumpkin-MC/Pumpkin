use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::{Nbt, NbtCompound, tag::NbtTag};
use serde_json::Value;

pub use pumpkin_gametest::model::{GameTestDefinition as TestInstance, GameTestRotation, TestType};
use tracing::warn;

pub type TestInstanceRegistry = HashMap<String, TestInstance>;

/// Loads all test instances embedded in the binary at compile time.
///
/// Ids are fully qualified, so entries here can be overridden by later
/// [`load_test_instances_from_dir`] calls for the same id.
pub fn load_embedded_test_instances(registry: &mut TestInstanceRegistry) -> usize {
    let before = registry.len();

    for &id in pumpkin_world::test_instance::all_names() {
        let Some(content) = pumpkin_world::test_instance::json(id) else {
            warn!("Embedded test instance '{id}' has no JSON payload");
            continue;
        };

        match serde_json::from_str::<TestInstance>(content) {
            Ok(instance) if instance.is_valid() => {
                registry.insert(id.to_owned(), instance);
            }
            Ok(_) => warn!("Embedded test instance '{id}' failed validation"),
            Err(e) => warn!("Failed to parse embedded test instance '{id}': {e}"),
        }
    }

    registry.len() - before
}

pub fn load_test_instances_from_dir(
    namespace: &str,
    test_instance_dir: &Path,
    registry: &mut TestInstanceRegistry,
) -> usize {
    if !test_instance_dir.is_dir() {
        return 0;
    }

    let before = registry.len();
    load_test_instances_recursive(namespace, test_instance_dir, test_instance_dir, registry);
    registry.len() - before
}

fn load_test_instances_recursive(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut TestInstanceRegistry,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            load_test_instances_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();

            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }

            let stem_path = stem_path.replace('\\', "/");
            let test_instance_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(instance) = serde_json::from_str::<TestInstance>(&content)
                && instance.is_valid()
            {
                registry.insert(test_instance_id, instance);
            }
        }
    }
}

/// Encodes the datapack test definition using the same map shape consumed by
/// `GameTestInstance.CODEC` / `TestData.CODEC` in vanilla.
///
/// `RegistryData` expects an unnamed NBT compound payload for each registry entry.
#[must_use]
pub fn to_registry_entry(entry_id: String, instance: &TestInstance) -> RegistryEntryData {
    let mut nbt = NbtCompound::new();
    nbt.put_string(
        "type",
        match instance.instance_type {
            TestType::BlockBased => "minecraft:block_based",
        }
        .to_string(),
    );
    nbt.put("environment", json_value_to_nbt(&instance.environment));
    nbt.put_string("structure", instance.structure.clone());
    nbt.put_int("max_ticks", instance.max_ticks);
    nbt.put_int("setup_ticks", instance.setup_ticks);
    nbt.put_bool("required", instance.required);
    nbt.put_string("rotation", instance.rotation.serialized_name().to_string());
    nbt.put_bool("manual_only", instance.manual_only);
    nbt.put_int("max_attempts", instance.max_attempts);
    nbt.put_int("required_successes", instance.required_successes);
    nbt.put_bool("sky_access", instance.sky_access);
    nbt.put_int("padding", instance.padding);

    RegistryEntryData {
        entry_id,
        data: Some(Nbt::from(nbt).write().to_vec().into_boxed_slice()),
    }
}

fn json_value_to_nbt(value: &Value) -> NbtTag {
    match value {
        Value::Null => NbtTag::String(String::new().into()),
        Value::Bool(value) => NbtTag::Byte(i8::from(*value)),
        Value::Number(value) => value.as_i64().map_or_else(
            || {
                value.as_u64().map_or_else(
                    || NbtTag::Double(value.as_f64().unwrap_or_default()),
                    |value| {
                        i32::try_from(value).map_or_else(
                            |_| {
                                i64::try_from(value)
                                    .map_or_else(|_| NbtTag::Double(value as f64), NbtTag::Long)
                            },
                            NbtTag::Int,
                        )
                    },
                )
            },
            |value| i32::try_from(value).map_or(NbtTag::Long(value), NbtTag::Int),
        ),
        Value::String(value) => NbtTag::String(value.clone().into()),
        Value::Array(values) => NbtTag::List(values.iter().map(json_value_to_nbt).collect()),
        Value::Object(values) => {
            let mut compound = NbtCompound::new();
            for (name, value) in values {
                compound.put(name, json_value_to_nbt(value));
            }
            NbtTag::Compound(compound)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn loads_block_based_test_instance_with_vanilla_defaults() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("mob_ai");

        fs::create_dir_all(&nested).expect("create nested test_instance dir");

        fs::write(
            nested.join("some_ai_test.json"),
            r#"{
                "type": "minecraft:block_based",
                "environment": "minecraft:default",
                "structure": "pumpkin:some_ai_test",
                "max_ticks": 200
            }"#,
        )
        .expect("write test instance");

        let mut registry = TestInstanceRegistry::new();
        let loaded = load_test_instances_from_dir("pumpkin", dir.path(), &mut registry);

        assert_eq!(loaded, 1);

        let instance = registry
            .get("pumpkin:mob_ai/some_ai_test")
            .expect("test instance should be registered");

        assert_eq!(instance.instance_type, TestType::BlockBased);
        assert_eq!(
            instance.environment,
            Value::String("minecraft:default".into())
        );
        assert_eq!(instance.structure, "pumpkin:some_ai_test");
        assert_eq!(instance.max_ticks, 200);
        assert_eq!(instance.setup_ticks, 0);
        assert!(instance.required);
        assert_eq!(instance.rotation, GameTestRotation::None);
        assert!(!instance.manual_only);
        assert_eq!(instance.max_attempts, 1);
        assert_eq!(instance.required_successes, 1);
        assert!(!instance.sky_access);
        assert_eq!(instance.padding, 0);
    }
}
