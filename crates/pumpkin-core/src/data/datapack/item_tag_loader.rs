use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

/// Collects the entries of every `tags/item` (or legacy `tags/items`) file of a namespace into
/// `tags`, keyed by `namespace:path`. Entries are item ids or `#tag` references.
///
/// `replace` is not honoured: the tags are merged with the generated ones, see
/// `pumpkin_data::dynamic_tag`.
pub fn load_item_tags_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    tags_dir: &Path,
    tags: &mut HashMap<String, Vec<String>, S>,
) {
    for sub in ["item", "items"] {
        let dir = tags_dir.join(sub);
        if dir.is_dir() {
            load_recursive(namespace, &dir, &dir, tags);
        }
    }
}

fn load_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    tags: &mut HashMap<String, Vec<String>, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_recursive(namespace, base_dir, &path, tags);
            continue;
        }
        let Some(stem) = path
            .strip_prefix(base_dir)
            .ok()
            .and_then(|rel| rel.to_str())
            .and_then(|rel| rel.strip_suffix(".json"))
        else {
            continue;
        };
        let tag_id = format!("{namespace}:{}", stem.replace('\\', "/"));
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(json) = serde_json::from_str::<Value>(&content)
        {
            tags.entry(tag_id)
                .or_default()
                .extend(parse_tag_values(&json));
        }
    }
}

/// Reads the `values` of a tag file. An entry is a string or `{"id": ..., "required": ...}`.
/// Entries that name unknown items are kept, they are resolved when the tag is checked.
fn parse_tag_values(json: &Value) -> Vec<String> {
    json.get("values")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| match value {
                    Value::String(id) => Some(id.clone()),
                    Value::Object(object) => {
                        object.get("id").and_then(Value::as_str).map(str::to_string)
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_and_object_entries() {
        let json: Value = serde_json::from_str(
            r##"{"replace": false, "values": ["lonsdaleite:gem", {"id": "minecraft:diamond", "required": false}, "#c:gems", 3]}"##,
        )
        .unwrap();
        assert_eq!(
            parse_tag_values(&json),
            ["lonsdaleite:gem", "minecraft:diamond", "#c:gems"]
        );
        assert!(parse_tag_values(&Value::Null).is_empty());
    }

    #[test]
    fn loads_nested_tag_files() {
        let dir = std::env::temp_dir().join(format!("pumpkin_item_tags_{}", std::process::id()));
        let nested = dir.join("item").join("ingots");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("lonsdaleite.json"),
            r#"{"values": ["lonsdaleite:ingot"]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("item").join("swords.json"),
            r#"{"values": ["lonsdaleite:sword"]}"#,
        )
        .unwrap();
        let mut tags = HashMap::new();
        load_item_tags_from_dir("c", &dir, &mut tags);
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(tags["c:ingots/lonsdaleite"], ["lonsdaleite:ingot"]);
        assert_eq!(tags["c:swords"], ["lonsdaleite:sword"]);
    }
}
