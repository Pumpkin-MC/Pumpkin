use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::file_loader::{collect_files, load_files_parallel};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DynamicTagEntry {
    pub id: String,
    pub values: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawTagFile {
    #[serde(default)]
    replace: bool,
    #[serde(default)]
    values: Vec<RawTagValue>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RawTagValue {
    String(String),
    Object { id: String },
}

pub fn load_tags_from_dir<S1: std::hash::BuildHasher + Default, S2: std::hash::BuildHasher>(
    namespace: &str,
    tags_dir: &Path,
    tags: &mut HashMap<String, HashMap<String, DynamicTagEntry, S1>, S2>,
) -> usize {
    let mut total_loaded = 0;
    let Ok(entries) = fs::read_dir(tags_dir) else {
        return 0;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cat_name = entry.file_name().to_string_lossy().to_string();
        if cat_name == "worldgen" {
            // Worldgen tags are nested, e.g. tags/worldgen/biome
            if let Ok(wg_entries) = fs::read_dir(&path) {
                for wg_entry in wg_entries.flatten() {
                    let wg_path = wg_entry.path();
                    if wg_path.is_dir() {
                        let sub_cat =
                            format!("worldgen/{}", wg_entry.file_name().to_string_lossy());
                        let category_map = tags.entry(sub_cat).or_default();
                        let before = category_map.len();
                        load_tag_category(namespace, &wg_path, category_map);
                        total_loaded += category_map.len() - before;
                    }
                }
            }
        } else {
            let category_map = tags.entry(cat_name).or_default();
            let before = category_map.len();
            load_tag_category(namespace, &path, category_map);
            total_loaded += category_map.len() - before;
        }
    }

    total_loaded
}

fn load_tag_category<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    category_map: &mut HashMap<String, DynamicTagEntry, S>,
) {
    let files = collect_files(base_dir, "json");
    let parsed = load_files_parallel(
        namespace,
        base_dir,
        &files,
        "json",
        |_id, _path, content| {
            let raw = serde_json::from_str::<RawTagFile>(content).ok()?;
            let mut string_values = Vec::with_capacity(raw.values.len());
            for val in raw.values {
                match val {
                    RawTagValue::String(s) => string_values.push(s),
                    RawTagValue::Object { id } => string_values.push(id),
                }
            }
            Some((raw.replace, string_values))
        },
    );

    for (tag_id, _rel, (replace, string_values)) in parsed {
        if replace {
            category_map.insert(
                tag_id.clone(),
                DynamicTagEntry {
                    id: tag_id,
                    values: string_values,
                },
            );
        } else {
            let entry = category_map
                .entry(tag_id.clone())
                .or_insert_with(|| DynamicTagEntry {
                    id: tag_id,
                    values: Vec::new(),
                });
            for val in string_values {
                if !entry.values.contains(&val) {
                    entry.values.push(val);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_load_tags() {
        let temp = tempfile::tempdir().expect("tempdir");
        let tags_dir = temp.path().join("tags");
        let block_tags = tags_dir.join("block");
        fs::create_dir_all(&block_tags).expect("create dir");
        fs::write(
            block_tags.join("custom_tag.json"),
            r#"{"replace": false, "values": ["minecraft:stone", {"id": "minecraft:dirt"}]}"#,
        )
        .expect("write");

        let mut map: HashMap<String, HashMap<String, DynamicTagEntry>> = HashMap::new();
        let loaded = load_tags_from_dir("custom", &tags_dir, &mut map);
        assert_eq!(loaded, 1);
        let block_map = map.get("block").expect("block tags");
        let entry = block_map.get("custom:custom_tag").expect("custom_tag");
        assert_eq!(entry.values, ["minecraft:stone", "minecraft:dirt"]);
    }
}
