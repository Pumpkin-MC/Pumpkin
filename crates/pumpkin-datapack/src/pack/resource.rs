use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;

/// Trait for reading resources from a datapack.
pub trait PackResources: Send + Sync {
    /// Read a root resource (e.g., `pack.mcmeta`).
    fn get_root_resource(&self, path: &str) -> Option<Vec<u8>>;

    /// Read a resource from `data/<namespace>/<path>`.
    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>>;

    /// List all resource paths under `data/<namespace>/<prefix>`.
    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String>;

    /// Return all namespaces this pack provides data for.
    fn get_namespaces(&self) -> Vec<String>;

    /// Unique pack identifier.
    fn pack_id(&self) -> &str;

    /// Configure overlay directories to check before the root data directory
    /// when resolving resources. Overlay directories are relative to the pack
    /// root (e.g. `overlay_81`). This method may be called after construction,
    /// e.g. when the repository computes overlays from `pack.mcmeta`.
    fn set_overlays(&self, overlays: Vec<String>);
}

/// Reads resources from a directory on disk.
pub struct PathPackResources {
    root: PathBuf,
    overlays: Mutex<Vec<String>>,
}

impl PathPackResources {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self {
            root,
            overlays: Mutex::new(Vec::new()),
        }
    }

    fn data_path(&self, namespace: &str, path: &str) -> PathBuf {
        self.root.join("data").join(namespace).join(path)
    }

    fn overlay_data_path(&self, overlay: &str, namespace: &str, path: &str) -> PathBuf {
        self.root
            .join(overlay)
            .join("data")
            .join(namespace)
            .join(path)
    }
}

impl PackResources for PathPackResources {
    fn get_root_resource(&self, path: &str) -> Option<Vec<u8>> {
        let full = self.root.join(path);
        std::fs::read(full).ok()
    }

    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>> {
        // Check overlay directories first (in order)
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let full = self.overlay_data_path(overlay, namespace, path);
            if let Ok(data) = std::fs::read(&full) {
                return Some(data);
            }
        }
        drop(overlays);
        // Fall back to root data directory
        let full = self.data_path(namespace, path);
        std::fs::read(full).ok()
    }

    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        let prefix_with_slash = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}/")
        };

        // Collect from overlay directories first
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let overlay_dir = self
                .root
                .join(overlay)
                .join("data")
                .join(namespace)
                .join(prefix);
            results.extend(Self::collect_resources_from_dir(
                &overlay_dir,
                &prefix_with_slash,
            ));
        }
        drop(overlays);

        // Collect from root data directory
        let root_dir = self.data_path(namespace, prefix);
        results.extend(Self::collect_resources_from_dir(
            &root_dir,
            &prefix_with_slash,
        ));

        results.sort();
        results.dedup();
        results
    }

    fn get_namespaces(&self) -> Vec<String> {
        let mut namespaces: Vec<String> = Vec::new();
        // Collect from overlay directories
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let overlay_data = self.root.join(overlay).join("data");
            if let Ok(dir) = std::fs::read_dir(&overlay_data) {
                for entry in dir.flatten() {
                    if entry.path().is_dir()
                        && let Some(name) = entry.file_name().to_str().map(String::from)
                        && !namespaces.contains(&name)
                    {
                        namespaces.push(name);
                    }
                }
            }
        }
        drop(overlays);
        // Collect from root
        let data_dir = self.root.join("data");
        if let Ok(dir) = std::fs::read_dir(&data_dir) {
            for entry in dir.flatten() {
                if entry.path().is_dir()
                    && let Some(name) = entry.file_name().to_str().map(String::from)
                    && !namespaces.contains(&name)
                {
                    namespaces.push(name);
                }
            }
        }
        namespaces
    }

    fn pack_id(&self) -> &str {
        self.root
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    }

    fn set_overlays(&self, overlays: Vec<String>) {
        *self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = overlays;
    }
}

impl PathPackResources {
    /// Recursively collect file paths from a directory, returning paths relative to the
    /// given `prefix_with_slash`. Deeper recursion appends `/name` segments.
    fn collect_resources_from_dir(dir: &std::path::Path, prefix_with_slash: &str) -> Vec<String> {
        let Ok(dir) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut results = Vec::new();
        for entry in dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    let child_prefix = format!("{prefix_with_slash}{name}/");
                    results.extend(Self::collect_resources_from_dir(&path, &child_prefix));
                }
            } else if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                results.push(format!("{prefix_with_slash}{name}"));
            }
        }
        results
    }
}

/// Reads resources from a `.zip` file.
pub struct ZipPackResources {
    path: PathBuf,
    files: HashMap<String, Vec<u8>>,
    namespaces: Vec<String>,
    overlays: Mutex<Vec<String>>,
}

impl ZipPackResources {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(&path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut files = HashMap::new();
        let mut namespaces_set: HashMap<String, bool> = HashMap::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_path = entry.name().to_string();
            if entry.is_dir() {
                continue;
            }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            files.insert(entry_path.clone(), buf);

            // Detect namespaces: data/<namespace>/...
            if let Some(rest) = entry_path.strip_prefix("data/")
                && let Some(ns) = rest.split('/').next()
            {
                namespaces_set.entry(ns.to_string()).or_insert(true);
            }
        }

        Ok(Self {
            path,
            files,
            namespaces: namespaces_set.into_keys().collect(),
            overlays: Mutex::new(Vec::new()),
        })
    }
}

impl PackResources for ZipPackResources {
    fn get_root_resource(&self, path: &str) -> Option<Vec<u8>> {
        self.files.get(path).cloned()
    }

    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>> {
        // Check overlay directories first (in order)
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let overlay_path = format!("{overlay}/data/{namespace}/{path}");
            if let Some(data) = self.files.get(&overlay_path) {
                return Some(data.clone());
            }
        }
        drop(overlays);
        // Fall back to root data directory
        let full_path = format!("data/{namespace}/{path}");
        self.files.get(&full_path).cloned()
    }

    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();

        // The prefix is expected to be included in the returned paths so that
        // callers can pass them back to get_resource() which constructs
        // "data/{namespace}/{path}". PathPackResources does this by collecting
        // from "data/{namespace}/{prefix}/" and prepending "{prefix}/" to each.
        // ZipPackResources must do the same for consistency.
        let strip_prefix = if prefix.is_empty() {
            format!("data/{namespace}/")
        } else {
            format!("data/{namespace}/{prefix}/")
        };
        let add_prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}/")
        };

        // Collect from overlay directories first
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let overlay_strip = if prefix.is_empty() {
                format!("{overlay}/data/{namespace}/")
            } else {
                format!("{overlay}/data/{namespace}/{prefix}/")
            };
            for key in self.files.keys() {
                if let Some(rest) = key.strip_prefix(&overlay_strip) {
                    results.push(format!("{add_prefix}{rest}"));
                }
            }
        }
        drop(overlays);

        // Collect from root data directory
        for key in self.files.keys() {
            if let Some(rest) = key.strip_prefix(&strip_prefix) {
                results.push(format!("{add_prefix}{rest}"));
            }
        }

        results.sort();
        results.dedup();
        results
    }

    fn get_namespaces(&self) -> Vec<String> {
        let mut namespaces: Vec<String> = Vec::new();

        // Collect from overlay directories
        let overlays = self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for overlay in overlays.iter() {
            let overlay_prefix = format!("{overlay}/data/");
            for key in self.files.keys() {
                if let Some(rest) = key.strip_prefix(&overlay_prefix)
                    && let Some(ns) = rest.split('/').next()
                {
                    let ns = ns.to_string();
                    if !namespaces.contains(&ns) {
                        namespaces.push(ns);
                    }
                }
            }
        }
        drop(overlays);

        // Collect from root
        for ns in &self.namespaces {
            if !namespaces.contains(ns) {
                namespaces.push(ns.clone());
            }
        }

        namespaces
    }

    fn pack_id(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    }

    fn set_overlays(&self, overlays: Vec<String>) {
        *self
            .overlays
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = overlays;
    }
}

/// Virtual pack that exposes Pumpkin's compiled-in static data as pack resources.
///
/// Uses the embedded vanilla datapack data generated by `pumpkin-codegen`.
/// This works in both dev and release builds - no filesystem reads at runtime.
pub struct VanillaPackResources;

impl PackResources for VanillaPackResources {
    fn get_root_resource(&self, path: &str) -> Option<Vec<u8>> {
        // We don't ship a pack.mcmeta for the vanilla pack; vanilla compatibility
        // is implicit.
        let _ = path;
        None
    }

    fn get_resource(&self, namespace: &str, path: &str) -> Option<Vec<u8>> {
        pumpkin_data::embedded_vanilla_datapack::get_vanilla_resource(namespace, path)
            .map(<[u8]>::to_vec)
    }

    fn list_resources(&self, namespace: &str, prefix: &str) -> Vec<String> {
        let paths =
            pumpkin_data::embedded_vanilla_datapack::list_vanilla_resources(namespace, prefix);
        paths.iter().map(|s| (*s).to_string()).collect()
    }

    fn get_namespaces(&self) -> Vec<String> {
        pumpkin_data::embedded_vanilla_datapack::get_vanilla_namespaces()
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    }

    fn pack_id(&self) -> &'static str {
        "vanilla"
    }

    fn set_overlays(&self, _overlays: Vec<String>) {
        // Vanilla pack does not support overlays.
    }
}
