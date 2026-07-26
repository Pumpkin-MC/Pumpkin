use super::{
    Context, LoadedPlugin, ManagerError, PLUGIN_DIR, Plugin, PluginManager, PluginMetadata,
    PluginState, cache, permissions,
};
use crate::LOGGER_IMPL;
use crate::plugin::loader::{LoaderError, PluginLoader};
use futures::future::join_all;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};

impl PluginManager {
    /// Add a new plugin loader implementation
    pub async fn add_loader(&self, loader: Arc<dyn PluginLoader>) {
        self.loaders.write().await.push(loader);

        // Try to load previously unloaded files with the new loader
        self.retry_unloaded_files().await;
    }

    /// Retry loading files that couldn't be loaded previously
    async fn retry_unloaded_files(&self) {
        let files_to_retry: Vec<PathBuf> =
            { self.unloaded_files.read().await.iter().cloned().collect() };
        let mut retry_tasks = Vec::new();

        for path in files_to_retry {
            if let Ok(task) = self.start_loading_plugin(&path).await {
                retry_tasks.push(task);
            }
        }

        // Wait for all retry tasks to complete
        join_all(retry_tasks).await;
    }

    /// Helper for topological sort of plugins based on dependencies
    fn topological_sort(plugins: &[(String, Vec<String>)]) -> Result<Vec<String>, String> {
        fn visit(
            name: &str,
            deps_map: &HashMap<String, &Vec<String>>,
            plugin_names: &HashSet<String>,
            visited: &mut HashSet<String>,
            current_path: &mut HashSet<String>,
            sorted: &mut Vec<String>,
        ) -> Result<(), String> {
            if current_path.contains(name) {
                return Err(format!(
                    "Circular dependency detected involving plugin: {name}"
                ));
            }
            if !visited.contains(name) {
                current_path.insert(name.to_string());
                if let Some(deps) = deps_map.get(name) {
                    for dep in *deps {
                        if !plugin_names.contains(dep) {
                            return Err(format!("Plugin {name} depends on missing plugin: {dep}"));
                        }
                        visit(dep, deps_map, plugin_names, visited, current_path, sorted)?;
                    }
                }
                current_path.remove(name);
                visited.insert(name.to_string());
                sorted.push(name.to_string());
            }
            Ok(())
        }
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = HashSet::new();
        let plugin_names: HashSet<String> = plugins.iter().map(|(n, _)| n.clone()).collect();
        let deps_map: HashMap<String, &Vec<String>> =
            plugins.iter().map(|(n, d)| (n.clone(), d)).collect();

        for (name, _) in plugins {
            visit(
                name,
                &deps_map,
                &plugin_names,
                &mut visited,
                &mut current_path,
                &mut sorted,
            )?;
        }

        Ok(sorted)
    }

    /// Ask the server owner if they allow the permissions requested by a plugin
    #[expect(clippy::print_stdout)]
    fn ask_permission_confirmation(metadata: &PluginMetadata) -> (bool, std::time::Duration) {
        use colored::Colorize;
        use rustyline::DefaultEditor;

        if metadata.permissions.is_empty() {
            return (true, std::time::Duration::ZERO);
        }

        let start_time = std::time::Instant::now();

        println!(
            "\n{} \"{}\" ({}) requests the following permissions:",
            "Plugin".bold(),
            metadata.name.cyan(),
            metadata.version.green()
        );
        for permission in &metadata.permissions {
            if let Some(description) = permissions::get_permission_description(permission) {
                println!(
                    "  - {}: {}",
                    permission.yellow().bold(),
                    description.italic()
                );
            } else {
                println!("  - {}", permission.yellow().bold());
            }
        }

        let prompt = format!(
            "\n{} [y/N]: ",
            "Do you want to allow these permissions and load the plugin?".bold()
        );

        let mut rl_taken = if let Some(logger_option) = crate::LOGGER_IMPL.get()
            && let Some((wrapper, _, _)) = logger_option
            && let Some(rl) = wrapper.take_readline()
        {
            Some((wrapper, rl))
        } else {
            None
        };

        let result = if let Some((_, ref mut rl)) = rl_taken {
            rl.readline(&prompt).is_ok_and(|line| {
                let input = line.trim().to_lowercase();
                input == "y" || input == "yes"
            })
        } else {
            let mut rl = DefaultEditor::new().expect("Failed to create rustyline editor");
            rl.readline(&prompt).is_ok_and(|line| {
                let input = line.trim().to_lowercase();
                input == "y" || input == "yes"
            })
        };

        if let Some((wrapper, rl)) = rl_taken {
            wrapper.return_readline(rl);
        }

        (result, start_time.elapsed())
    }

    /// Spawn initialization for a single plugin
    #[expect(clippy::too_many_lines)]
    async fn spawn_plugin_initialization(
        &self,
        mut instance: Box<dyn Plugin>,
        metadata: PluginMetadata,
        loader_data: Box<dyn Any + Send + Sync>,
        loader: Arc<dyn PluginLoader>,
        path: PathBuf,
    ) -> Result<tokio::task::JoinHandle<()>, ManagerError> {
        // Mark plugin as loading
        self.plugin_states
            .write()
            .await
            .insert(metadata.name.clone(), PluginState::Loading);

        let self_ref = self
            .self_ref
            .read()
            .await
            .clone()
            .ok_or(ManagerError::ServerNotInitialized)?;

        let context = Arc::new(Context::new(
            metadata.clone(),
            Arc::clone(
                &self
                    .server
                    .read()
                    .await
                    .clone()
                    .ok_or(ManagerError::ServerNotInitialized)?,
            ),
            Arc::clone(&self.handlers),
            Arc::clone(&self_ref),
            Arc::clone(&LOGGER_IMPL),
        ));

        // Create the plugin structure first
        let plugin = LoadedPlugin {
            metadata: metadata.clone(),
            instance: None, // Will be set after successful initialization
            loader: loader.clone(),
            loader_data: Some(loader_data),
            is_active: false, // Will be set to true after successful initialization
            context: context.clone(),
            path,
        };

        let plugin_index = {
            let mut plugins = self.plugins.write().await;
            plugins.push(plugin);
            plugins.len() - 1
        };

        // Spawn async task for plugin initialization
        let self_ref_clone = Arc::clone(&self_ref);
        let state_notify = Arc::clone(&self.state_notify);
        let plugin_name = metadata.name.clone();
        let loader_clone = loader.clone();

        let task = tokio::spawn(async move {
            // Initialize the plugin
            match instance.on_load(context.clone()).await {
                Ok(()) => {
                    // Update plugin state to loaded
                    {
                        let mut plugins = self_ref_clone.plugins.write().await;
                        if let Some(plugin) = plugins.get_mut(plugin_index) {
                            plugin.instance = Some(instance);
                            plugin.is_active = true;
                        }
                    }
                    self_ref_clone
                        .plugin_states
                        .write()
                        .await
                        .insert(plugin_name.clone(), PluginState::Loaded);
                    state_notify.notify_waiters();

                    info!("Loaded {} ({})", metadata.name, metadata.version);

                    if !metadata.permissions.is_empty() {
                        warn!(
                            "Plugin \"{}\" uses the following permissions: {:?}",
                            metadata.name, metadata.permissions
                        );
                    }
                }
                Err(e) => {
                    // Handle initialization failure
                    let error_msg = format!("Initialization failed: {e}");
                    let _ = instance.on_unload(context).await;

                    // Get the loader data before removing the plugin
                    let loader_data: Option<Box<dyn Any + Send + Sync>> = {
                        let mut plugins = self_ref_clone.plugins.write().await;
                        if let Some(plugin) = plugins.get_mut(plugin_index) {
                            plugin.loader_data.take()
                        } else {
                            None
                        }
                    };

                    // Try to unload the plugin data
                    if let Some(data) = loader_data {
                        tokio::spawn(async move {
                            loader_clone.unload(data).await.ok();
                        });
                    }

                    {
                        let mut plugins = self_ref_clone.plugins.write().await;
                        if plugin_index < plugins.len() {
                            plugins.remove(plugin_index);
                        }
                    };
                    self_ref_clone
                        .plugin_states
                        .write()
                        .await
                        .insert(plugin_name.clone(), PluginState::Failed(error_msg.clone()));
                    state_notify.notify_waiters();

                    error!("Failed to initialize plugin {plugin_name}: {error_msg}",);
                }
            }
        });

        Ok(task)
    }

    /// Load all plugins from the plugin directory
    pub async fn load_plugins(&self) -> Result<std::time::Duration, ManagerError> {
        let path = Path::new(PLUGIN_DIR);

        if !path.exists() {
            std::fs::create_dir(path)?;
            return Ok(std::time::Duration::ZERO);
        }

        let cache_path = path.join("permission_cache.json");
        let mut cache = cache::PermissionCache::load(&cache_path).await;

        let mut prepared_plugins = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("deactivated"))
            {
                continue;
            }

            // Find a loader that can handle this file
            let loaders = self.loaders.read().await;
            let mut loader_found = false;
            for loader in loaders.iter() {
                if loader.can_load(&path) {
                    match loader.load(&path).await {
                        Ok((instance, metadata, loader_data)) => {
                            prepared_plugins.push((
                                instance,
                                metadata,
                                loader_data,
                                loader.clone(),
                                path.clone(),
                            ));
                            loader_found = true;
                        }
                        Err(err) => error!("Failed to load plugin from {:?}: {}", path, err),
                    }
                    break;
                }
            }

            if !loader_found {
                self.unloaded_files.write().await.insert(path.clone());
            }
        }

        // Resolve dependencies
        let metadata_list: Vec<(String, Vec<String>)> = prepared_plugins
            .iter()
            .map(|(_, m, _, _, _)| (m.name.clone(), m.dependencies.clone()))
            .collect();

        let sorted_names =
            Self::topological_sort(&metadata_list).map_err(ManagerError::DependencyError)?;

        // Map names back to prepared plugins
        #[expect(clippy::type_complexity)]
        let mut plugins_map: HashMap<
            String,
            (
                Box<dyn Plugin>,
                PluginMetadata,
                Box<dyn Any + Send + Sync>,
                Arc<dyn PluginLoader>,
                PathBuf,
            ),
        > = prepared_plugins
            .into_iter()
            .map(|(i, m, d, l, p)| (m.name.clone(), (i, m, d, l, p)))
            .collect();

        let mut total_wait_time = std::time::Duration::ZERO;

        for name in sorted_names {
            if let Some((instance, metadata, loader_data, loader, path)) = plugins_map.remove(&name)
            {
                let (allowed, wait_time) = self
                    .check_permissions_cached(&path, &metadata, &mut cache, &cache_path)
                    .await;

                total_wait_time += wait_time;

                if !allowed {
                    warn!(
                        "Permission denied for plugin \"{}\", skipping loading.",
                        metadata.name
                    );
                    continue;
                }

                match self
                    .spawn_plugin_initialization(instance, metadata, loader_data, loader, path)
                    .await
                {
                    Ok(task) => {
                        // We must await each initialization to ensure dependencies are ready
                        if let Err(err) = task.await {
                            error!("Plugin initialization task panicked: {}", err);
                        }
                    }
                    Err(err) => error!("{}", err),
                }
            }
        }

        Ok(total_wait_time)
    }

    async fn check_permissions_cached(
        &self,
        path: &Path,
        metadata: &PluginMetadata,
        cache: &mut cache::PermissionCache,
        cache_path: &Path,
    ) -> (bool, std::time::Duration) {
        let hash = cache::calculate_hash(path).await.unwrap_or_default();

        if let Some(entry) = cache.entries.get(&hash)
            && entry.permissions_requested == metadata.permissions
        {
            info!(
                "Found cached permission decision for plugin \"{}\" (approved: {})",
                metadata.name, entry.approved
            );
            return (entry.approved, std::time::Duration::ZERO);
        }

        let (allowed, wait_time) = Self::ask_permission_confirmation(metadata);
        cache.entries.insert(
            hash,
            cache::PermissionCacheEntry {
                permissions_requested: metadata.permissions.clone(),
                approved: allowed,
            },
        );
        let _ = cache.save(cache_path).await;
        (allowed, wait_time)
    }

    /// Start loading a plugin asynchronously
    pub(super) async fn start_loading_plugin(
        &self,
        path: &Path,
    ) -> Result<tokio::task::JoinHandle<()>, ManagerError> {
        for loader in self.loaders.read().await.iter() {
            if loader.can_load(path) {
                let (instance, metadata, loader_data) = loader.load(path).await?;

                let cache_path = Path::new(PLUGIN_DIR).join("permission_cache.json");
                let mut cache = cache::PermissionCache::load(&cache_path).await;

                let (allowed, _) = self
                    .check_permissions_cached(path, &metadata, &mut cache, &cache_path)
                    .await;

                if !allowed {
                    warn!(
                        "Permission denied for plugin \"{}\", skipping loading.",
                        metadata.name
                    );
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        "Permission denied".to_string(),
                    )));
                }

                return self
                    .spawn_plugin_initialization(
                        instance,
                        metadata,
                        loader_data,
                        loader.clone(),
                        path.to_path_buf(),
                    )
                    .await;
            }
        }

        // No loader could handle this file, track it for future attempts
        self.unloaded_files.write().await.insert(path.to_path_buf());

        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Attempt to load a single plugin file
    pub async fn try_load_plugin(&self, path: &Path) -> Result<(), ManagerError> {
        self.start_loading_plugin(path).await?.await.map_err(|e| {
            ManagerError::LoaderError(LoaderError::InitializationFailed(format!(
                "Task join error: {e}"
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn topological_sort() {
        let plugins = vec![
            ("A".to_string(), vec!["B".to_string()]),
            ("B".to_string(), vec!["C".to_string()]),
            ("C".to_string(), vec![]),
        ];
        let sorted = PluginManager::topological_sort(&plugins).unwrap();
        assert_eq!(sorted, vec!["C", "B", "A"]);

        let plugins_complex = vec![
            ("A".to_string(), vec!["B".to_string(), "C".to_string()]),
            ("B".to_string(), vec!["D".to_string()]),
            ("C".to_string(), vec!["D".to_string()]),
            ("D".to_string(), vec![]),
        ];
        let sorted = PluginManager::topological_sort(&plugins_complex).unwrap();
        // Multiple valid sorts possible, but D must be before B and C, and B, C must be before A.
        assert_eq!(sorted[0], "D");
        assert!(sorted[1] == "B" || sorted[1] == "C");
        assert!(sorted[2] == "B" || sorted[2] == "C");
        assert_eq!(sorted[3], "A");

        let plugins_circular = vec![
            ("A".to_string(), vec!["B".to_string()]),
            ("B".to_string(), vec!["A".to_string()]),
        ];
        assert!(PluginManager::topological_sort(&plugins_circular).is_err());

        let plugins_missing = vec![("A".to_string(), vec!["B".to_string()])];
        assert!(PluginManager::topological_sort(&plugins_missing).is_err());
    }
}
