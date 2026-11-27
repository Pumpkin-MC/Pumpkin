use crate::entity::player::advancement::predicate::CriterionConditions;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::warn;

use super::file_loader::{collect_files, load_files_parallel};

/// A parsed dynamic advancement definition from a datapack with pre-parsed criteria.
#[derive(Clone, Debug)]
pub struct DynamicAdvancementDefinition {
    pub id: String,
    pub json: String,
    pub criteria: HashMap<String, (String, Arc<CriterionConditions>)>,
}

/// Loads all advancement JSON files from a directory recursively into `registry`.
pub fn load_advancements_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, DynamicAdvancementDefinition, S>,
) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    let before = registry.len();
    let files = collect_files(dir, "json");
    let loaded = load_files_parallel(namespace, dir, &files, "json", |entry_id, path, content| {
        let Ok(val) = serde_json::from_str::<Value>(content) else {
            warn!("Failed to parse advancement JSON at {}", path.display());
            return None;
        };

        let mut criteria = HashMap::new();
        if let Some(crit_obj) = val.get("criteria").and_then(Value::as_object) {
            for (name, c_val) in crit_obj {
                let trigger = c_val
                    .get("trigger")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let cond_str = c_val
                    .get("conditions")
                    .map_or_else(|| "{}".to_string(), ToString::to_string);
                let parsed = Arc::new(CriterionConditions::parse(&trigger, &cond_str));
                criteria.insert(name.clone(), (trigger, parsed));
            }
        }

        Some(DynamicAdvancementDefinition {
            id: entry_id.to_string(),
            json: content.to_string(),
            criteria,
        })
    });

    for (entry_id, _rel, definition) in loaded {
        registry.insert(entry_id, definition);
    }
    registry.len() - before
}
