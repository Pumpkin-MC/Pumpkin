use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::ResourceManager;
use crate::pack::resource::PackResources;

/// Per-namespace layered resource lookup.
/// Packs are stored low-to-high priority; the last pack wins.
struct NamespaceManager {
    /// The namespace this manager handles.
    namespace: String,
    /// (`pack_id`, `pack_resources`) in priority order (low -> high).
    packs: Vec<(String, Arc<dyn PackResources>)>,
}

impl NamespaceManager {
    fn new(namespace: String) -> Self {
        Self {
            namespace,
            packs: Vec::new(),
        }
    }

    fn add_pack(&mut self, id: String, resources: Arc<dyn PackResources>) {
        self.packs.push((id, resources));
    }

    fn get_resource(&self, path: &str) -> Option<Vec<u8>> {
        // Iterate reverse: highest priority last
        for (_, pack) in self.packs.iter().rev() {
            if let Some(data) = pack.get_resource(&self.namespace, path) {
                return Some(data);
            }
        }
        None
    }

    fn get_resource_stack(&self, path: &str) -> Vec<(String, Vec<u8>)> {
        let mut stack = Vec::new();
        for (id, pack) in &self.packs {
            if let Some(data) = pack.get_resource(&self.namespace, path) {
                stack.push((id.clone(), data));
            }
        }
        stack
    }

    fn list_resources(&self, prefix: &str) -> Vec<String> {
        let mut all = Vec::new();
        let mut seen = HashSet::new();
        for (_, pack) in &self.packs {
            for path in pack.list_resources(&self.namespace, prefix) {
                if seen.insert(path.clone()) {
                    all.push(path);
                }
            }
        }
        all
    }
}

/// Combines multiple packs into a single namespace-aware resource manager.
/// Lower-index packs have lower priority (overridden by higher-index packs).
pub struct MultiPackResourceManager {
    namespaces: HashMap<String, NamespaceManager>,
    all_namespaces: Vec<String>,
}

impl MultiPackResourceManager {
    /// Build from an ordered list of pack resources (low -> high priority).
    #[must_use]
    pub fn new(packs: &[Arc<dyn PackResources>]) -> Self {
        let mut ns_map: HashMap<String, NamespaceManager> = HashMap::new();
        let mut all_ns_set = HashSet::new();

        for pack in packs {
            let id = pack.pack_id().to_string();
            let namespaces = pack.get_namespaces();
            for ns in namespaces {
                all_ns_set.insert(ns.clone());
                let entry = ns_map
                    .entry(ns.clone())
                    .or_insert_with(|| NamespaceManager::new(ns));
                entry.add_pack(id.clone(), Arc::clone(pack));
            }
        }

        // Also store the pack as a fallback for resource listing across namespaces
        let mut all_ns: Vec<String> = all_ns_set.into_iter().collect();
        all_ns.sort();

        Self {
            namespaces: ns_map,
            all_namespaces: all_ns,
        }
    }
}

impl ResourceManager for MultiPackResourceManager {
    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>> {
        self.namespaces
            .get(namespace)
            .and_then(|ns| ns.get_resource(path))
    }

    fn get_resource_stack(&self, namespace: &str, path: &str) -> Vec<(String, Vec<u8>)> {
        self.namespaces
            .get(namespace)
            .map(|ns| ns.get_resource_stack(path))
            .unwrap_or_default()
    }

    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String> {
        self.namespaces
            .get(namespace)
            .map(|ns| ns.list_resources(prefix))
            .unwrap_or_default()
    }

    fn get_namespaces(&self) -> Vec<String> {
        self.all_namespaces.clone()
    }
}
