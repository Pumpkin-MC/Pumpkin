use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::file_loader::{collect_files, load_files_parallel};

pub fn load_item_modifiers_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    modifiers: &mut HashMap<String, Value, S>,
) -> usize {
    let files = collect_files(dir, "json");
    let loaded = load_files_parallel(namespace, dir, &files, "json", |_id, _path, content| {
        serde_json::from_str::<Value>(content).ok()
    });
    let count = loaded.len();
    for (entry_id, _rel, val) in loaded {
        modifiers.insert(entry_id, val);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
