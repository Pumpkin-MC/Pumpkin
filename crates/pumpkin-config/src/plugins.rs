use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Plugin system configuration.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct PluginsConfig {
    /// Whether the plugin system is enabled. If false, no plugins will be loaded.
    pub enabled: bool,
    /// Whether to watch the plugins directory and automatically hot-reload modified plugins.
    pub hot_reload: bool,
    /// Whether the server asks for confirmation in the console when a plugin requests new permissions.
    pub ask_permission_confirmation: bool,
    /// Whether to allow loading unsigned WASM plugins.
    pub allow_unsigned: bool,
    /// List of permissions that are globally pre-approved for all plugins (bypassing confirmation).
    pub allowed_permissions: Vec<String>,
    /// List of permissions that are globally blocked for all plugins.
    pub blocked_permissions: Vec<String>,
    /// Whether host environment variables are inherited into WASI environments by default without explicit permissions.
    pub inherit_env: bool,
    /// Whether network sockets in plugins are restricted to localhost/loopback by default.
    pub loopback_only: bool,
    /// Optional global maximum memory limit in megabytes (MB) per plugin instance.
    /// If not set, memory is only constrained by the host system's available memory.
    pub max_memory_mb: Option<u64>,
    /// Per-plugin configuration and overrides, keyed by plugin name.
    ///
    /// Each entry is named after the plugin it applies to (for example `my_plugin`).
    /// You only need to list the plugins you actually want to configure or override.
    ///
    /// Example:
    ///
    /// ```toml
    /// [plugins.overrides.my_plugin]
    /// enabled = true
    /// allow_unsigned = true
    /// max_memory_mb = 128
    /// allowed_permissions = ["fs:read:data", "fs:write:data"]
    /// blocked_permissions = ["network:outbound"]
    /// loopback_only = true
    ///
    /// [plugins.overrides.my_plugin.environment]
    /// MY_API_KEY = "secret_key"
    /// ```
    pub overrides: HashMap<String, PluginOverride>,
    /// Whether Pumpkin should verify WASM plugin signatures before loading.
    pub verify_signatures: bool,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hot_reload: false,
            ask_permission_confirmation: true,
            allow_unsigned: true,
            allowed_permissions: Vec::new(),
            blocked_permissions: Vec::new(),
            inherit_env: false,
            loopback_only: false,
            max_memory_mb: None,
            overrides: HashMap::new(),
            verify_signatures: true,
        }
    }
}

/// Settings for a single plugin, letting a server owner turn it off or change
/// its permissions, unsigned execution policy, memory limit, or environment variables.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct PluginOverride {
    /// Whether this specific plugin is enabled. If set to `false`, the plugin will be ignored during loading.
    pub enabled: bool,
    /// Override whether this plugin is allowed to run if unsigned.
    pub allow_unsigned: Option<bool>,
    /// Optional maximum memory limit in megabytes (MB) for this specific plugin.
    /// Overrides the global `max_memory_mb` setting if specified.
    pub max_memory_mb: Option<u64>,
    /// Permissions pre-approved specifically for this plugin (skips interactive confirmation).
    pub allowed_permissions: Vec<String>,
    /// Additional permissions blocked specifically for this plugin.
    pub blocked_permissions: Vec<String>,
    /// Override whether network access is restricted to loopback for this plugin.
    pub loopback_only: Option<bool>,
    /// Custom environment variables passed directly to this plugin's WASI environment.
    pub environment: HashMap<String, String>,
}

impl Default for PluginOverride {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_unsigned: None,
            max_memory_mb: None,
            allowed_permissions: Vec::new(),
            blocked_permissions: Vec::new(),
            loopback_only: None,
            environment: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = PluginsConfig::default();
        assert!(config.enabled);
        assert!(!config.hot_reload);
        assert!(config.ask_permission_confirmation);
        assert!(config.allow_unsigned);
        assert!(config.allowed_permissions.is_empty());
        assert!(config.blocked_permissions.is_empty());
        assert!(!config.inherit_env);
        assert!(!config.loopback_only);
        assert_eq!(config.max_memory_mb, None);
        assert!(config.overrides.is_empty());
        assert!(config.verify_signatures);
    }

    #[test]
    fn parse_toml_overrides() {
        let toml_str = r#"
            enabled = true
            hot_reload = true
            ask_permission_confirmation = false
            allow_unsigned = false
            allowed_permissions = ["fs:read:data"]
            blocked_permissions = ["network:outbound"]
            inherit_env = true
            loopback_only = true
            max_memory_mb = 256
            verify_signatures = false

            [overrides.my_plugin]
            enabled = false
            allow_unsigned = true
            max_memory_mb = 128
            allowed_permissions = ["network:tcp"]
            blocked_permissions = ["fs:write:data"]
            loopback_only = false

            [overrides.my_plugin.environment]
            API_KEY = "12345"
        "#;

        let config: PluginsConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert!(config.hot_reload);
        assert!(!config.ask_permission_confirmation);
        assert!(!config.allow_unsigned);
        assert_eq!(config.allowed_permissions, vec!["fs:read:data"]);
        assert_eq!(config.blocked_permissions, vec!["network:outbound"]);
        assert!(config.inherit_env);
        assert!(config.loopback_only);
        assert_eq!(config.max_memory_mb, Some(256));
        assert!(!config.verify_signatures);

        let override_cfg = config.overrides.get("my_plugin").unwrap();
        assert!(!override_cfg.enabled);
        assert_eq!(override_cfg.allow_unsigned, Some(true));
        assert_eq!(override_cfg.max_memory_mb, Some(128));
        assert_eq!(override_cfg.allowed_permissions, vec!["network:tcp"]);
        assert_eq!(override_cfg.blocked_permissions, vec!["fs:write:data"]);
        assert_eq!(override_cfg.loopback_only, Some(false));
        assert_eq!(override_cfg.environment.get("API_KEY").unwrap(), "12345");
    }
}
