pub mod converter;
pub mod manager;

/// A resource manager provides layered read access to all enabled packs.
pub trait ResourceManager: Send + Sync {
    /// Get the highest-priority version of a resource.
    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>>;

    /// Get all versions of a resource (from lowest to highest priority).
    fn get_resource_stack(&self, namespace: &str, path: &str) -> Vec<(String, Vec<u8>)>;

    /// List all resource paths under `data/<namespace>/<prefix>`.
    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String>;

    /// Get all namespaces available across all packs.
    fn get_namespaces(&self) -> Vec<String>;

    /// Check if a resource exists.
    fn has_resource(&self, namespace: &str, path: &str) -> bool {
        self.get_resource(namespace, path).is_some()
    }
}
