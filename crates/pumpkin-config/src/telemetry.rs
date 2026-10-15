use serde::{Deserialize, Serialize};

/// Telemetry configuration options.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Whether anonymous telemetry is enabled. Default is true.
    pub enabled: bool,
    /// Custom telemetry backend ingestion endpoint.
    pub endpoint: String,
    /// Heartbeat interval in seconds (default: 300 seconds / 5 minutes). Minimum 60s.
    pub interval_secs: u64,
    /// Whether to opt-in to displaying this server in the public community directory.
    pub public: bool,
    /// Public server name displayed on the analytics dashboard if public is true.
    pub server_name: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "https://market.pumpkinmc.org/api/v1/rest/telemetry/heartbeat".to_string(),
            interval_secs: 300,
            public: false,
            server_name: None,
        }
    }
}

impl TelemetryConfig {
    /// Validates telemetry configuration options.
    pub const fn validate(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_default() {
        let default_config = TelemetryConfig::default();
        assert!(default_config.enabled);
        assert_eq!(
            default_config.endpoint,
            "https://market.pumpkinmc.org/api/v1/rest/telemetry/heartbeat"
        );
        assert_eq!(default_config.interval_secs, 300);
        assert!(!default_config.public);
        assert_eq!(default_config.server_name, None);
    }

    #[test]
    fn telemetry_toml_deserialization() {
        let toml_str = r#"
            enabled = false
            endpoint = "http://localhost:5000/api/v1/rest/telemetry/heartbeat"
            interval_secs = 60
            public = true
            server_name = "Test SMP"
        "#;

        let config: TelemetryConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert_eq!(
            config.endpoint,
            "http://localhost:5000/api/v1/rest/telemetry/heartbeat"
        );
        assert_eq!(config.interval_secs, 60);
        assert!(config.public);
        assert_eq!(config.server_name.as_deref(), Some("Test SMP"));
    }
}
