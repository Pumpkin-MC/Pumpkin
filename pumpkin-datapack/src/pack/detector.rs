use std::path::Path;
use std::sync::Arc;

use super::Pack;
use super::metadata::PackMcmeta;
use super::resource::{PackResources, PathPackResources, ZipPackResources};

/// Detects whether a path is a valid datapack (folder or zip) and creates a `Pack`.
pub struct PackDetector;

impl PackDetector {
    /// Try to create a `Pack` from a path (folder or `.zip`).
    /// Returns `None` if the path is not a valid pack.
    #[must_use]
    pub fn detect_pack(path: &Path) -> Option<Pack> {
        let (resources, mcmeta): (Arc<dyn super::resource::PackResources>, PackMcmeta) = if path
            .is_dir()
        {
            let mcmeta_path = path.join("pack.mcmeta");
            if !mcmeta_path.exists() {
                tracing::debug!("Skipping dir (no pack.mcmeta): {}", path.display());
                return None;
            }
            let raw = match std::fs::read(&mcmeta_path) {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!("Failed to read pack.mcmeta at {}: {e}", path.display());
                    return None;
                }
            };
            let mcmeta: PackMcmeta = match serde_json::from_slice(&raw) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to parse pack.mcmeta at {}: {e}", path.display());
                    return None;
                }
            };

            let res = PathPackResources::new(path.to_path_buf());
            let overlays = mcmeta.matching_overlay_dirs();
            if !overlays.is_empty() {
                res.set_overlays(overlays);
            }
            (Arc::new(res), mcmeta)
        } else if path.extension().and_then(|e| e.to_str()) == Some("zip") {
            let zip_res = match ZipPackResources::new(path.to_path_buf()) {
                Ok(z) => z,
                Err(e) => {
                    tracing::warn!(
                        "Failed to unzip datapack '{}': {e}. \
                             Try extracting the zip manually into a folder, \
                             or delete the .zip if you already have the folder.",
                        path.file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default()
                    );
                    return None;
                }
            };
            let Some(mcmeta_bytes) = zip_res.get_root_resource("pack.mcmeta") else {
                tracing::debug!("Skipping zip (no pack.mcmeta): {}", path.display());
                return None;
            };
            let mcmeta: PackMcmeta = match serde_json::from_slice(&mcmeta_bytes) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to parse pack.mcmeta in zip {}: {e}", path.display());
                    return None;
                }
            };

            let overlays = mcmeta.matching_overlay_dirs();
            if !overlays.is_empty() {
                zip_res.set_overlays(overlays);
            }
            (Arc::new(zip_res), mcmeta)
        } else {
            tracing::trace!("Skipping non-pack file: {}", path.display());
            return None;
        };

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let id = format!("file/{name}");
        let compatibility = mcmeta.compatibility();

        let feature_flags = mcmeta
            .features
            .as_ref()
            .map(|f| f.enabled.clone())
            .unwrap_or_default();

        tracing::info!("Detected datapack: {id} (compat={compatibility:?})");

        Some(Pack {
            id,
            name,
            resources: Some(resources),
            metadata: Some(Box::new(mcmeta)),
            source: super::PackSource::World,
            compatibility,
            feature_flags,
        })
    }
}
