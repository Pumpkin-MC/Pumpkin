use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

pub fn load_item_modifiers_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    modifiers: &mut HashMap<String, Value, S>,
) -> usize {
    let mut loaded = 0;
    load_item_modifiers_recursive(namespace, dir, dir, modifiers, &mut loaded);
    loaded
}

fn load_item_modifiers_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    modifiers: &mut HashMap<String, Value, S>,
    loaded: &mut usize,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_item_modifiers_recursive(namespace, base_dir, &path, modifiers, loaded);
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
            let modifier_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(val) = serde_json::from_str::<Value>(&content)
            {
                modifiers.insert(modifier_id, val);
                *loaded += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_load_item_modifiers() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mod_dir = temp.path().join("item_modifier");
        fs::create_dir_all(&mod_dir).expect("create dir");
        fs::write(
            mod_dir.join("test_mod.json"),
            r#"{"function": "minecraft:set_count", "count": 5}"#,
        )
        .expect("write");

        let mut map = HashMap::new();
        let loaded = load_item_modifiers_from_dir("custom", &mod_dir, &mut map);
        assert_eq!(loaded, 1);
        assert!(map.contains_key("custom:test_mod"));
    }
}
