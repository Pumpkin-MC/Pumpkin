use std::collections::HashMap;

use crate::Identifier;
use crate::resource::ResourceManager;

/// A parsed predicate (used for loot conditions, advancements, etc.).
/// Stores the full raw JSON for runtime evaluation.
#[derive(Debug, Clone)]
pub struct Predicate {
    pub id: Identifier,
    pub data: serde_json::Value,
}

/// A parsed item modifier.
#[derive(Debug, Clone)]
pub struct ItemModifier {
    pub id: Identifier,
    pub functions: Vec<serde_json::Value>,
}

/// Load predicates from datapacks.
pub fn load_predicates(
    manager: &dyn ResourceManager,
) -> Result<HashMap<Identifier, Predicate>, crate::DatapackError> {
    let mut predicates = HashMap::new();

    for ns in manager.get_namespaces() {
        let paths =
            crate::resource::list_resources_multi(manager, &ns, &["predicate", "predicates"]);
        for path in &paths {
            if !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let Some(data) = manager.get_resource(&ns, path) else {
                continue;
            };
            let raw: serde_json::Value = serde_json::from_slice(&data)?;

            let pred_name = path
                .strip_prefix("predicate/")
                .or_else(|| path.strip_prefix("predicates/"))
                .and_then(|p| p.strip_suffix(".json"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(ns.clone(), pred_name.to_string())?;

            predicates.insert(id.clone(), Predicate { id, data: raw });
        }
    }

    Ok(predicates)
}

/// Load item modifiers from datapacks.
pub fn load_item_modifiers(
    manager: &dyn ResourceManager,
) -> Result<HashMap<Identifier, ItemModifier>, crate::DatapackError> {
    let mut modifiers = HashMap::new();

    for ns in manager.get_namespaces() {
        let paths = crate::resource::list_resources_multi(
            manager,
            &ns,
            &["item_modifier", "item_modifiers"],
        );
        for path in &paths {
            if !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let Some(data) = manager.get_resource(&ns, path) else {
                continue;
            };
            // Item modifiers can be either an array of functions or a single function object
            let functions = if let Ok(arr) = serde_json::from_slice::<Vec<serde_json::Value>>(&data)
            {
                arr
            } else {
                // Single object: wrap in a Vec
                let single: serde_json::Value = serde_json::from_slice(&data)?;
                vec![single]
            };

            let mod_name = path
                .strip_prefix("item_modifier/")
                .or_else(|| path.strip_prefix("item_modifiers/"))
                .and_then(|p| p.strip_suffix(".json"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(ns.clone(), mod_name.to_string())?;

            modifiers.insert(id.clone(), ItemModifier { id, functions });
        }
    }

    Ok(modifiers)
}
