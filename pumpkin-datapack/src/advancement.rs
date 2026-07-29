use std::collections::HashMap;

use crate::DatapackError;
use crate::Identifier;
use crate::resource::ResourceManager;

/// A parsed advancement file from a datapack.
/// The raw JSON is preserved for forward compatibility; the advancement
/// evaluation system can deserialize it further when needed.
#[derive(Debug, Clone)]
pub struct AdvancementFile {
    pub id: Identifier,
    pub data: serde_json::Value,
}

/// Load all advancement JSON files from datapacks.
///
/// Scans `data/<namespace>/advancement/` for `.json` files across all
/// enabled packs and returns a map of advancement ID to parsed data.
pub fn load_advancements(
    manager: &dyn ResourceManager,
) -> Result<HashMap<Identifier, AdvancementFile>, DatapackError> {
    let mut advancements = HashMap::new();

    for ns in manager.get_namespaces() {
        let paths = manager.list_resources(&ns, "advancement");
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

            let adv_path = path
                .strip_prefix("advancement/")
                .and_then(|p| p.strip_suffix(".json"))
                .or_else(|| path.strip_suffix(".json"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(&ns, adv_path);

            advancements.insert(id.clone(), AdvancementFile { id, data: raw });
        }
    }

    Ok(advancements)
}
