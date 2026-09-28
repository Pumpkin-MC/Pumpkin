use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use serde_json::Value;
use tracing::warn;

pub type ContextProviderRegistry = HashMap<String, NbtTag>;

#[must_use]
pub fn json_value_to_nbt(value: &Value) -> NbtTag {
    match value {
        Value::Null => NbtTag::String(String::new().into()),
        Value::Bool(val) => NbtTag::Byte(i8::from(*val)),
        Value::Number(val) => val.as_i64().map_or_else(
            || {
                val.as_u64().map_or_else(
                    || NbtTag::Double(val.as_f64().unwrap_or_default()),
                    |u_val| {
                        i32::try_from(u_val).map_or_else(
                            |_| {
                                #[allow(clippy::cast_precision_loss)]
                                i64::try_from(u_val)
                                    .map_or_else(|_| NbtTag::Double(u_val as f64), NbtTag::Long)
                            },
                            NbtTag::Int,
                        )
                    },
                )
            },
            |i_val| i32::try_from(i_val).map_or(NbtTag::Long(i_val), NbtTag::Int),
        ),
        Value::String(val) => NbtTag::String(val.clone().into()),
        Value::Array(values) => NbtTag::List(values.iter().map(json_value_to_nbt).collect()),
        Value::Object(values) => {
            let mut compound = NbtCompound::new();
            for (name, val) in values {
                compound.put(name, json_value_to_nbt(val));
            }
            NbtTag::Compound(compound)
        }
    }
}

/// Loads all embedded context int and float providers into their respective registries.
pub fn load_embedded_context_providers(
    int_registry: &mut ContextProviderRegistry,
    float_registry: &mut ContextProviderRegistry,
) -> (usize, usize) {
    let int_before = int_registry.len();
    for &id in pumpkin_data::context_provider::all_context_int_provider_names() {
        if let Some(content) = pumpkin_data::context_provider::get_context_int_provider_json(id)
            && let Ok(val) = serde_json::from_str::<Value>(content)
        {
            let tag = json_value_to_nbt(&val);
            int_registry.insert(id.to_string(), tag);
        }
    }
    let int_count = int_registry.len() - int_before;

    let float_before = float_registry.len();
    for &id in pumpkin_data::context_provider::all_context_float_provider_names() {
        if let Some(content) = pumpkin_data::context_provider::get_context_float_provider_json(id)
            && let Ok(val) = serde_json::from_str::<Value>(content)
        {
            let tag = json_value_to_nbt(&val);
            float_registry.insert(id.to_string(), tag);
        }
    }
    let float_count = float_registry.len() - float_before;

    (int_count, float_count)
}

pub fn load_context_providers_from_dir(
    namespace: &str,
    provider_dir: &Path,
    registry: &mut ContextProviderRegistry,
) -> usize {
    if !provider_dir.is_dir() {
        return 0;
    }

    let before = registry.len();
    load_context_providers_recursive(namespace, provider_dir, provider_dir, registry);
    registry.len() - before
}

fn load_context_providers_recursive(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut ContextProviderRegistry,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            load_context_providers_recursive(namespace, base_dir, &path, registry);
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
            let provider_id = format!("{namespace}:{stem_path}");

            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(val) => {
                        let tag = json_value_to_nbt(&val);
                        registry.insert(provider_id, tag);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse context provider '{provider_id}' from '{}': {e}",
                            path.display()
                        );
                    }
                },
                Err(e) => {
                    warn!(
                        "Failed to read context provider file '{}': {e}",
                        path.display()
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_value_to_nbt_works() {
        let json: Value = serde_json::json!({
            "type": "minecraft:add",
            "left": 10,
            "right": 20.5,
            "items": ["a", "b"]
        });

        let tag = json_value_to_nbt(&json);
        if let NbtTag::Compound(compound) = tag {
            assert_eq!(compound.get_string("type"), Some("minecraft:add"));
            assert_eq!(compound.get_int("left"), Some(10));
            assert_eq!(compound.get_double("right"), Some(20.5));
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn loads_embedded_context_providers() {
        let mut int_reg = HashMap::new();
        let mut float_reg = HashMap::new();
        let (ints, floats) = load_embedded_context_providers(&mut int_reg, &mut float_reg);
        assert!(ints > 0);
        assert!(floats > 0);
        assert!(int_reg.contains_key("minecraft:compostable/low"));
        assert!(float_reg.contains_key("minecraft:cooking/speed_default"));
    }
}
