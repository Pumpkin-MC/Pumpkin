use super::registry::TagRegistry;
use super::{TagEntry, TagFile};
use crate::DatapackError;
use crate::Identifier;
use crate::resource::ResourceManager;

/// Load all tags from the resource manager.
pub fn load_tags(manager: &dyn ResourceManager) -> Result<TagRegistry, DatapackError> {
    let mut registry = TagRegistry::new();

    // Discover tag directories by scanning "tags/" under each namespace
    let namespaces = manager.get_namespaces();

    // First pass: which registry directories exist?
    let mut registry_dirs: Vec<String> = Vec::new();
    for ns in &namespaces {
        for path in manager.list_resources(ns, "tags") {
            // path is like "block/..." or "item/..."
            if let Some(dir) = path.split('/').next()
                && !registry_dirs.contains(&dir.to_string())
            {
                registry_dirs.push(dir.to_string());
            }
        }
    }

    // Second pass: load tags for each registry
    for registry_name in &registry_dirs {
        let directory = format!("tags/{registry_name}");
        for ns in &namespaces {
            for path in manager.list_resources(ns, &directory) {
                let Some(data) = manager.get_resource(ns, &path) else {
                    continue;
                };

                // Parse the tag file
                let tag_file: TagFile =
                    serde_json::from_slice(&data).map_err(DatapackError::Json)?;

                // Compute tag ID from path
                let tag_path = path
                    .strip_prefix(&format!("tags/{registry_name}/"))
                    .and_then(|p| p.strip_suffix(".json"))
                    .or_else(|| path.strip_suffix(".json"))
                    .unwrap_or(path.as_str());
                let tag_id = Identifier::new(ns.clone(), tag_path.to_string())?;

                // Parse values
                let mut entries = Vec::new();
                for val in &tag_file.values {
                    if let Some(s) = val.as_str() {
                        if let Some(stripped) = s.strip_prefix('#') {
                            if let Ok(ref_id) = Identifier::parse(stripped) {
                                entries.push(TagEntry::TagReference(ref_id, true));
                            }
                        } else if let Ok(elem_id) = Identifier::parse(s) {
                            entries.push(TagEntry::Element(elem_id, true));
                        }
                    } else if let Some(obj) = val.as_object() {
                        let id_str = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let required = obj
                            .get("required")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(true);
                        if let Some(stripped) = id_str.strip_prefix('#') {
                            if let Ok(ref_id) = Identifier::parse(stripped) {
                                entries.push(TagEntry::TagReference(ref_id, required));
                            }
                        } else if let Ok(elem_id) = Identifier::parse(id_str) {
                            entries.push(TagEntry::Element(elem_id, required));
                        }
                    }
                }

                if tag_file.replace {
                    registry.add_or_replace(registry_name, tag_id, entries);
                } else {
                    registry.add_or_append(registry_name, tag_id, entries);
                }
            }
        }
    }

    // Resolve all tag references
    registry.resolve_all()?;

    Ok(registry)
}
