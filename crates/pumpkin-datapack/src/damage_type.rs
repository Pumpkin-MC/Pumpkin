use std::collections::HashMap;

use crate::DatapackError;
use crate::Identifier;
use crate::resource::ResourceManager;

/// A parsed damage type from a datapack.
#[derive(Debug, Clone)]
pub struct DamageTypeFile {
    pub id: Identifier,
    pub data: serde_json::Value,
}

impl DamageTypeFile {
    /// Parse the serialized `DamageType` from the raw JSON.
    /// Fields: `message_id`, `exhaustion`, `scaling`, `effects`, `death_message_type`
    #[must_use]
    pub fn exhaustion(&self) -> f32 {
        self.data
            .get("exhaustion")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0) as f32
    }

    #[must_use]
    pub fn message_id(&self) -> &str {
        self.data
            .get("message_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("generic")
    }

    #[must_use]
    pub fn scaling(&self) -> &str {
        self.data
            .get("scaling")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("never")
    }

    #[must_use]
    pub fn effects(&self) -> Option<&str> {
        self.data.get("effects").and_then(serde_json::Value::as_str)
    }

    #[must_use]
    pub fn death_message_type(&self) -> Option<&str> {
        self.data
            .get("death_message_type")
            .and_then(serde_json::Value::as_str)
    }
}

/// Load all damage type JSON files from datapacks.
pub fn load_damage_types(
    manager: &dyn ResourceManager,
) -> Result<HashMap<Identifier, DamageTypeFile>, DatapackError> {
    let mut types = HashMap::new();

    for ns in manager.get_namespaces() {
        let paths =
            crate::resource::list_resources_multi(manager, &ns, &["damage_type", "damage_types"]);
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

            let type_path = path
                .strip_prefix("damage_type/")
                .or_else(|| path.strip_prefix("damage_types/"))
                .and_then(|p| p.strip_suffix(".json"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(ns.clone(), type_path.to_string())?;

            types.insert(id.clone(), DamageTypeFile { id, data: raw });
        }
    }

    Ok(types)
}
