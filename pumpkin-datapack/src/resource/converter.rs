use super::ResourceManager;
use crate::Identifier;

/// Maps between resource filesystem paths and logical resource IDs.
///
/// e.g., prefix="advancements", ext=".json"
///   `id_to_file("minecraft:story/root`") -> "advancements/story/root.json"
///   `file_to_id("advancements/story/root.json`") -> Some(("minecraft", "story/root"))
pub struct FileToIdConverter {
    pub prefix: String,
    pub extension: String,
}

impl FileToIdConverter {
    #[must_use]
    pub const fn new(prefix: String, extension: String) -> Self {
        Self { prefix, extension }
    }

    /// Convert a resource ID to its filesystem path (relative to the namespace's data dir).
    #[must_use]
    pub fn id_to_file(&self, id: &Identifier) -> String {
        format!("{}/{}{}", self.prefix, id.path(), self.extension)
    }

    /// Try to convert a file path back to a resource ID.
    /// Returns `Some((namespace, path_part))` or `None` if it doesn't match the prefix/extension.
    #[must_use]
    pub fn file_to_id(&self, file_path: &str) -> Option<(String, String)> {
        let file_path = file_path.strip_prefix(&self.prefix)?;
        let file_path = file_path.strip_prefix('/')?;
        file_path
            .strip_suffix(&self.extension)
            .map(|path| ("minecraft".to_string(), path.to_string()))
    }

    /// List all matching resources from the resource manager, returning a map of ID -> data.
    pub fn list_matching_resources(
        &self,
        manager: &dyn ResourceManager,
    ) -> Vec<(Identifier, Vec<u8>)> {
        let mut results = Vec::new();
        for namespace in manager.get_namespaces() {
            let prefix = &self.prefix;
            for path in manager.list_resources(&namespace, prefix) {
                if let Some(stripped) = path.strip_suffix(&self.extension) {
                    let id_path = stripped.to_string();
                    if let Ok(id) = Identifier::new(namespace.clone(), id_path)
                        && let Some(data) = manager.get_resource(&namespace, &path)
                    {
                        results.push((id, data));
                    }
                }
            }
        }
        results
    }
}
