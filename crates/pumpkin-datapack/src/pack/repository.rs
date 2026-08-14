use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::Pack;
use super::format::PackCompatibility;
use super::resource::PackResources;
use super::source::{FolderRepositorySource, RepositorySource, VanillaSource};

/// Central repository managing available and selected packs.
pub struct PackRepository {
    /// The world's `datapacks/` folder path.
    datapacks_folder: Arc<Path>,
    /// All discovered packs by ID.
    available: HashMap<String, Pack>,
    /// IDs of currently selected (enabled) packs.
    selected: Vec<String>,
}

impl PackRepository {
    #[must_use]
    pub fn new(world_path: &Path) -> Self {
        let datapacks_folder = world_path.join("datapacks");
        Self {
            datapacks_folder: datapacks_folder.into(),
            available: HashMap::new(),
            selected: vec!["vanilla".to_string()],
        }
    }

    /// Re-discover all packs from all sources.
    pub fn reload(&mut self) {
        self.available.clear();

        // Vanilla source
        let vanilla = VanillaSource;
        for pack in vanilla.load_packs() {
            self.available.insert(pack.id.clone(), pack);
        }

        // Folder source - scan world/datapacks/
        let folder = FolderRepositorySource::new(&self.datapacks_folder);
        for pack in folder.load_packs() {
            self.available.insert(pack.id.clone(), pack);
        }

        // Prune selected IDs that are no longer available
        self.selected.retain(|id| self.available.contains_key(id));
    }

    /// Configure selection based on an enabled/disabled list (from level.dat).
    pub fn configure(&mut self, enabled: &[String], disabled: &[String], safe_mode: bool) {
        if safe_mode {
            self.selected = vec!["vanilla".to_string()];
            return;
        }

        let mut selected = Vec::new();

        // Start with explicitly enabled packs that are available
        for id in enabled {
            if self.available.contains_key(id) && !selected.contains(id) {
                selected.push(id.clone());
            }
        }

        // Auto-discover new packs that should be automatically added
        for pack in self.available.values() {
            let id = &pack.id;
            if !disabled.contains(id)
                && !selected.contains(id)
                && pack.source.should_add_automatically()
                && pack.compatibility == PackCompatibility::Compatible
            {
                selected.push(id.clone());
            }
        }

        self.selected = selected;
    }

    /// Enable a pack by ID. Returns false if not found.
    pub fn add_pack(&mut self, id: &str) -> bool {
        if !self.available.contains_key(id) {
            return false;
        }
        if !self.selected.contains(&id.to_string()) {
            self.selected.push(id.to_string());
        }
        true
    }

    /// Enable a pack at a specific index position. Returns false if not found.
    pub fn add_pack_at(&mut self, id: &str, index: usize) -> bool {
        if !self.available.contains_key(id) {
            return false;
        }
        // Remove existing occurrence first
        self.selected.retain(|s| s != id);
        let index = index.min(self.selected.len());
        self.selected.insert(index, id.to_string());
        true
    }

    /// Disable a pack by ID.
    pub fn remove_pack(&mut self, id: &str) {
        self.selected.retain(|s| s != id);
    }

    /// Get a list of all available pack IDs.
    #[must_use]
    pub fn available_ids(&self) -> Vec<String> {
        self.available.keys().cloned().collect()
    }

    /// Get a list of selected pack IDs.
    #[must_use]
    pub fn selected_ids(&self) -> &[String] {
        &self.selected
    }
    /// Collect all feature flags from all enabled packs.
    #[must_use]
    pub fn enabled_feature_flags(&self) -> Vec<String> {
        let mut flags = vec!["minecraft:vanilla".to_string()];
        for id in &self.selected {
            if let Some(pack) = self.available.get(id) {
                for f in &pack.feature_flags {
                    let qualified = if f.contains(':') {
                        f.clone()
                    } else {
                        format!("minecraft:{f}")
                    };
                    if !flags.contains(&qualified) {
                        flags.push(qualified);
                    }
                }
            }
        }
        flags
    }
    /// Get a pack by ID.
    #[must_use]
    pub fn get_pack(&self, id: &str) -> Option<&Pack> {
        self.available.get(id)
    }

    /// Open all selected packs into resource readers, in the correct order
    /// (vanilla first, then file packs in the order they were enabled).
    ///
    /// For each pack that has metadata with overlay entries, the matching
    /// overlay directories (those whose format range includes
    /// `PackFormat::CURRENT`) are configured on the pack resources.
    #[must_use]
    pub fn open_all_selected(&self) -> Vec<Arc<dyn PackResources>> {
        let mut resources: Vec<Arc<dyn PackResources>> = Vec::new();
        for id in &self.selected {
            if let Some(pack) = self.available.get(id)
                && let Some(res) = &pack.resources
            {
                // Apply overlay directories from pack.mcmeta metadata
                if let Some(metadata) = &pack.metadata {
                    let overlays = metadata.matching_overlay_dirs();
                    if !overlays.is_empty() {
                        res.set_overlays(overlays);
                    }
                }
                resources.push(Arc::clone(res));
            }
        }
        resources
    }

    /// Get the enabled/disabled lists for saving to level.dat.
    #[must_use]
    pub fn to_config_lists(&self) -> (Vec<String>, Vec<String>) {
        let enabled = self.selected.clone();
        let disabled: Vec<String> = self
            .available
            .keys()
            .filter(|id| !self.selected.contains(id))
            .cloned()
            .collect();
        (enabled, disabled)
    }
}
