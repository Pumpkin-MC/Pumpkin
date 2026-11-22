use serde::{Deserialize, Serialize};

/// Configuration for checking for new Pumpkin updates on startup.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct UpdateCheckerConfig {
    /// Whether to check for new Pumpkin updates on server startup.
    pub enabled: bool,
    /// Whether to notify server operators in chat when they join if an update is available.
    pub notify_ops_on_join: bool,
    /// Connection timeout in milliseconds.
    pub connect_timeout: u32,
    /// Read timeout in milliseconds.
    pub read_timeout: u32,
}

impl Default for UpdateCheckerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notify_ops_on_join: true,
            connect_timeout: 5000,
            read_timeout: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_checker_default() {
        let config = UpdateCheckerConfig::default();
        assert!(config.enabled);
        assert!(config.notify_ops_on_join);
        assert_eq!(config.connect_timeout, 5000);
        assert_eq!(config.read_timeout, 5000);
    }

    #[test]
    fn update_checker_toml_deserialization() {
        let toml_str = r"
            enabled = false
            notify_ops_on_join = false
            connect_timeout = 3000
            read_timeout = 8000
        ";

        let config: UpdateCheckerConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert!(!config.notify_ops_on_join);
        assert_eq!(config.connect_timeout, 3000);
        assert_eq!(config.read_timeout, 8000);
    }
}
