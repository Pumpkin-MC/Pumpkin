use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::file_loader::{collect_files, load_files_parallel};

pub fn load_slot_sources_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    slot_sources: &mut HashMap<String, Value, S>,
) -> usize {
    let files = collect_files(dir, "json");
    let loaded = load_files_parallel(namespace, dir, &files, "json", |_id, _path, content| {
        serde_json::from_str::<Value>(content).ok()
    });
    let count = loaded.len();
    for (entry_id, _rel, val) in loaded {
        slot_sources.insert(entry_id, val);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn uppercase_extensions_retain_their_suffix() {
        for (name, id) in [
            ("armor.json", "custom:armor"),
            ("armor.JSON", "custom:armor.JSON"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join(name), "{}").unwrap();
            let mut map = HashMap::new();
            assert_eq!(
                load_slot_sources_from_dir("custom", dir.path(), &mut map),
                1
            );
            assert!(map.contains_key(id));
        }
    }

    #[test]
    fn parse_and_load_slot_sources() {
        let temp = tempfile::tempdir().expect("tempdir");
        let slot_dir = temp.path().join("slot_source");
        fs::create_dir_all(&slot_dir).expect("create dir");
        fs::write(
            slot_dir.join("test_slot.json"),
            r#"{"source": "minecraft:armor"}"#,
        )
        .expect("write");

        let mut map = HashMap::new();
        let loaded = load_slot_sources_from_dir("custom", &slot_dir, &mut map);
        assert_eq!(loaded, 1);
        assert!(map.contains_key("custom:test_slot"));
    }
}
