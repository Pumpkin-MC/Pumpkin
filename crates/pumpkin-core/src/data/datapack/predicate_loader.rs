use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

pub fn load_predicates_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    predicates: &mut HashMap<String, Value, S>,
) -> usize {
    let mut loaded = 0;
    load_predicates_recursive(namespace, dir, dir, predicates, &mut loaded);
    loaded
}

fn load_predicates_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    predicates: &mut HashMap<String, Value, S>,
    loaded: &mut usize,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_predicates_recursive(namespace, base_dir, &path, predicates, loaded);
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
            let predicate_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(val) = serde_json::from_str::<Value>(&content)
            {
                predicates.insert(predicate_id, val);
                *loaded += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
