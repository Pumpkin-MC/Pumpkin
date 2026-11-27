use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::file_loader::{collect_files, load_files_parallel};

pub fn load_predicates_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    predicates: &mut HashMap<String, Value, S>,
) -> usize {
    let files = collect_files(dir, "json");
    let loaded = load_files_parallel(namespace, dir, &files, "json", |_id, _path, content| {
        serde_json::from_str::<Value>(content).ok()
    });
    let count = loaded.len();
    for (entry_id, _rel, val) in loaded {
        predicates.insert(entry_id, val);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_and_load_predicates() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pred_dir = temp.path().join("predicate");
        fs::create_dir_all(&pred_dir).expect("create dir");
        fs::write(
            pred_dir.join("test_pred.json"),
            r#"{"type": "minecraft:match_tool"}"#,
        )
        .expect("write");

        let mut map = HashMap::new();
        let loaded = load_predicates_from_dir("custom", &pred_dir, &mut map);
        assert_eq!(loaded, 1);
        assert!(map.contains_key("custom:test_pred"));
    }
}
