use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);

/// Configuration for creating a new server instance.
///
/// This struct allows specifying per-instance overrides for configuration paths,
/// world directories, plugin directories, and network settings. Any field set to
/// `None` will fall back to the corresponding value in the `BasicConfiguration` or
/// `AdvancedConfiguration`.
pub struct InstanceConfig {
    /// The basic server configuration.
    pub basic: BasicConfiguration,
    /// The advanced server configuration.
    pub advanced: AdvancedConfiguration,
    /// The directory from which to load the configuration file.
    ///
    /// Defaults to the current working directory if `None`.
    pub config_dir: Option<PathBuf>,
    /// Override for the world directory path.
    ///
    /// If `None`, the world path is derived from `basic.default_level_name`.
    pub world_dir: Option<PathBuf>,
    /// Override for the plugin directory path.
    ///
    /// If `None`, defaults to `./plugins`.
    pub plugin_dir: Option<PathBuf>,
    /// Override for the data directory path.
    ///
    /// If `None`, defaults to `./data`.
    pub data_dir: Option<PathBuf>,
}

impl std::fmt::Debug for InstanceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // BasicConfiguration / AdvancedConfiguration do not implement Debug.
        f.debug_struct("InstanceConfig")
            .field("config_dir", &self.config_dir)
            .field("world_dir", &self.world_dir)
            .field("plugin_dir", &self.plugin_dir)
            .field("data_dir", &self.data_dir)
            .finish_non_exhaustive()
    }
}

impl InstanceConfig {
    pub fn new(basic: BasicConfiguration, advanced: AdvancedConfiguration) -> Self {
        Self {
            basic,
            advanced,
            config_dir: None,
            world_dir: None,
            plugin_dir: None,
            data_dir: None,
        }
    }

    pub fn with_config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir = Some(path);
        self
    }

    pub fn with_world_dir(mut self, path: PathBuf) -> Self {
        self.world_dir = Some(path);
        self
    }

    pub fn with_plugin_dir(mut self, path: PathBuf) -> Self {
        self.plugin_dir = Some(path);
        self
    }

    pub fn with_data_dir(mut self, path: PathBuf) -> Self {
        self.data_dir = Some(path);
        self
    }

    pub fn world_path(&self) -> PathBuf {
        self.world_dir
            .clone()
            .unwrap_or_else(|| self.basic.get_world_path())
    }

    pub fn plugin_path(&self) -> PathBuf {
        self.plugin_dir.clone().unwrap_or_else(|| PathBuf::from("./plugins"))
    }

    pub fn data_path(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| PathBuf::from("./data"))
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.clone().unwrap_or_else(|| PathBuf::from("."))
    }
}
