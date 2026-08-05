use std::path::Path;
use std::sync::Arc;

use super::Pack;
use super::detector::PackDetector;

/// A source that discovers packs (e.g., filesystem folder, built-in).
pub trait RepositorySource: Send + Sync {
    fn load_packs(&self) -> Vec<Pack>;
}

/// Scans a directory (typically `world/datapacks/`) for packs.
pub struct FolderRepositorySource {
    folder: Arc<Path>,
}

impl FolderRepositorySource {
    #[must_use]
    pub fn new(folder: &Path) -> Self {
        Self {
            folder: folder.into(),
        }
    }
}

impl RepositorySource for FolderRepositorySource {
    fn load_packs(&self) -> Vec<Pack> {
        let mut packs = Vec::new();
        let Ok(dir) = std::fs::read_dir(&self.folder) else {
            return packs;
        };

        // Collect all entries first to detect duplicates
        let entries: Vec<_> = dir.flatten().map(|e| e.path()).collect();

        // If a .zip file has a corresponding extracted folder, skip the zip
        // to avoid "Compression method not supported" errors and duplicate detection.
        let has_folder_for_zip = |zip_path: &Path| -> bool {
            let stem = zip_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            entries.iter().any(|p| {
                p.is_dir()
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s == stem)
            })
        };

        for path in &entries {
            let is_zip = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "zip");
            if is_zip && has_folder_for_zip(path) {
                tracing::debug!("Skipping zip (extracted folder exists): {}", path.display());
                continue;
            }
            if let Some(pack) = PackDetector::detect_pack(path) {
                packs.push(pack);
            }
        }
        packs
    }
}

/// Provides the built-in "vanilla" pack.
pub struct VanillaSource;

impl RepositorySource for VanillaSource {
    fn load_packs(&self) -> Vec<Pack> {
        vec![Pack::vanilla()]
    }
}
