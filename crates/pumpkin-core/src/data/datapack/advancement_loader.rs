use crate::entity::player::advancement::predicate::CriterionConditions;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::warn;

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
    load_advancements_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_advancements_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, DynamicAdvancementDefinition, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_advancements_recursive(namespace, base_dir, &path, registry);
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
            let adv_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
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

                    registry.insert(
                        adv_id.clone(),
                        DynamicAdvancementDefinition {
                            id: adv_id,
                            json: content,
                            criteria,
                        },
                    );
                } else {
                    warn!("Failed to parse advancement JSON at {}", path.display());
                }
            }
        }
    }
}
