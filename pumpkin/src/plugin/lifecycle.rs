use super::{ManagerError, PLUGIN_DIR, PluginManager, PluginMetadata, PluginState};
use crate::plugin::loader::LoaderError;
use notify::{EventKind, RecursiveMode, Watcher, event::ModifyKind};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, error, info};

impl PluginManager {
    /// Unload all loaded plugins
    pub async fn unload_all_plugins(&self) -> Result<(), ManagerError> {
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .filter(|p| p.is_active)
                .map(|p| p.metadata.name.clone())
                .collect()
        };

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                error!("Failed to unload plugin {name}: {e}");
            }
        }

        Ok(())
    }

    /// Start watching the plugins directory for changes
    pub async fn start_watcher(&self) -> Result<(), ManagerError> {
        if self.hot_reload_task.read().await.is_some() {
            return Ok(());
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .map_err(|e| ManagerError::IoError(std::io::Error::other(e)))?;

        let plugin_dir = Path::new(PLUGIN_DIR);
        if !plugin_dir.exists() {
            std::fs::create_dir_all(plugin_dir)?;
        }

        watcher
            .watch(plugin_dir, RecursiveMode::NonRecursive)
            .map_err(|e| ManagerError::IoError(std::io::Error::other(e)))?;

        let self_ref = self
            .self_ref
            .read()
            .await
            .clone()
            .ok_or(ManagerError::ManagerNotInitialized)?;

        let task = tokio::spawn(async move {
            // Keep watcher alive by moving it into the task
            let _watcher = watcher;

            while let Some(event) = rx.recv().await {
                if !self_ref
                    .hot_reload_enabled
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    continue;
                }

                match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(_) => {
                        for path in event.paths {
                            if path.extension().is_some_and(|ext| ext == "wasm") {
                                debug!("Detected change in plugin: {:?}", path);
                                // Give it a small delay to ensure file is completely written
                                tokio::time::sleep(Duration::from_millis(100)).await;

                                // We need to find if this plugin is already loaded to unload it first
                                let plugin_name = {
                                    let plugins = self_ref.plugins.read().await;
                                    plugins
                                        .iter()
                                        .find(|p| p.path == path)
                                        .map(|p| p.metadata.name.clone())
                                };

                                if let Some(name) = plugin_name {
                                    info!("Hot-reloading plugin: {}", name);
                                    let _ = self_ref.unload_plugin(&name).await;
                                }

                                // For now, we just try to load it. If it's already loaded,
                                // the loader might handle it or we might get a duplicate.
                                // Most WASM loaders will just create a new instance.
                                if let Err(e) = self_ref.start_loading_plugin(&path).await {
                                    error!("Failed to hot-reload plugin {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        *self.hot_reload_task.write().await = Some(task);
        self.set_hot_reload_enabled(true);
        Ok(())
    }

    /// Stop watching the plugins directory for changes
    pub async fn stop_watcher(&self) {
        let mut task_lock = self.hot_reload_task.write().await;
        if let Some(handle) = task_lock.take() {
            handle.abort();
        }
        self.set_hot_reload_enabled(false);
    }

    pub fn set_hot_reload_enabled(&self, enabled: bool) {
        self.hot_reload_enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_hot_reload_enabled(&self) -> bool {
        self.hot_reload_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Wait for a plugin to finish loading
    pub async fn wait_for_plugin(&self, plugin_name: &str) -> Result<(), ManagerError> {
        loop {
            let state = self.plugin_states.read().await.get(plugin_name).cloned();
            if let Some(state) = state {
                match state {
                    PluginState::Loaded => return Ok(()),
                    PluginState::Failed(error) => {
                        return Err(ManagerError::LoaderError(
                            LoaderError::InitializationFailed(error),
                        ));
                    }
                    PluginState::Loading => {
                        // Wait for state change notification
                        self.state_notify.notified().await;
                        continue;
                    }
                }
            }
            return Err(ManagerError::PluginNotFound(plugin_name.to_string()));
        }
    }

    /// Get the current state of a plugin
    pub async fn get_plugin_state(&self, plugin_name: &str) -> Option<PluginState> {
        self.plugin_states.read().await.get(plugin_name).cloned()
    }

    /// Checks if plugin active
    #[must_use]
    pub async fn is_plugin_active(&self, name: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .any(|p| p.metadata.name == name && p.is_active && p.instance.is_some())
    }

    /// Get list of active plugins
    #[must_use]
    pub async fn active_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .filter(|p| p.is_active && p.instance.is_some())
            .map(|p| p.metadata.clone())
            .collect()
    }

    /// Checks if plugin loaded
    #[must_use]
    pub async fn is_plugin_loaded(&self, name: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.iter().any(|p| p.metadata.name == name)
    }

    /// Get list of loaded plugins
    #[must_use]
    pub async fn loaded_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.iter().map(|p| p.metadata.clone()).collect()
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&self, name: &str) -> Result<(), ManagerError> {
        let index = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .position(|p| p.metadata.name == name)
                .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?
        };

        let mut plugin = {
            let mut plugins = self.plugins.write().await;
            plugins.remove(index)
        };

        if let Some(mut instance) = plugin.instance.take() {
            instance.on_unload(plugin.context.clone()).await.ok();
        }

        if plugin.loader.can_unload() {
            if let Some(data) = plugin.loader_data {
                plugin.loader.unload(data).await?;
            }
        } else {
            plugin.is_active = false;
            self.plugins.write().await.push(plugin);
        }

        // Remove from plugin states
        self.plugin_states.write().await.remove(name);

        Ok(())
    }

    /// Get all plugins that are currently loading
    pub async fn get_loading_plugins(&self) -> Vec<String> {
        let plugin_states = self.plugin_states.read().await;
        plugin_states
            .iter()
            .filter(|(_, state)| matches!(state, PluginState::Loading))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all plugins that failed to load
    pub async fn get_failed_plugins(&self) -> Vec<(String, String)> {
        let plugin_states = self.plugin_states.read().await;
        plugin_states
            .iter()
            .filter_map(|(name, state)| {
                if let PluginState::Failed(error) = state {
                    Some((name.clone(), error.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if all plugins have finished loading (either succeeded or failed)
    pub async fn all_plugins_loaded(&self) -> bool {
        let plugin_states = self.plugin_states.read().await;
        !plugin_states
            .values()
            .any(|state| matches!(state, PluginState::Loading))
    }

    /// Wait for all plugins to finish loading
    pub async fn wait_for_all_plugins(&self) {
        while !self.all_plugins_loaded().await {
            self.state_notify.notified().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_reload_flag_toggles() {
        let manager = PluginManager::new();
        assert!(!manager.is_hot_reload_enabled());
        manager.set_hot_reload_enabled(true);
        assert!(manager.is_hot_reload_enabled());
        manager.set_hot_reload_enabled(false);
        assert!(!manager.is_hot_reload_enabled());
    }

    #[tokio::test]
    async fn empty_manager_reports_no_plugins() {
        let manager = PluginManager::new();
        assert!(manager.all_plugins_loaded().await);
        assert!(!manager.is_plugin_loaded("missing").await);
        assert!(!manager.is_plugin_active("missing").await);
        assert!(manager.loaded_plugins().await.is_empty());
        assert!(manager.active_plugins().await.is_empty());
        assert!(manager.get_plugin_state("missing").await.is_none());
        assert!(manager.get_loading_plugins().await.is_empty());
        assert!(manager.get_failed_plugins().await.is_empty());
        assert!(matches!(
            manager.wait_for_plugin("missing").await,
            Err(ManagerError::PluginNotFound(_))
        ));
    }
}
