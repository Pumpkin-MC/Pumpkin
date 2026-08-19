use serde::{Deserialize, Serialize};

/// Configuration for client packet rate limiting.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct PacketLimiterConfig {
    /// Whether the packet rate limiter is enabled.
    pub enabled: bool,
    /// Maximum number of incoming packets allowed per second per client.
    /// Values <= 0.0 disable the rate limit.
    #[serde(alias = "max-packet-rate")]
    pub max_packet_rate: f64,
    /// Burst allowance capacity for packet rate limiting.
    #[serde(alias = "burst-capacity")]
    pub burst_capacity: f64,
    /// Kick message when a client exceeds the packet rate limit.
    #[serde(alias = "kick-message")]
    pub kick_message: String,
}

impl Default for PacketLimiterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_packet_rate: 500.0,
            burst_capacity: 500.0,
            kick_message: "Kicked for spamming packets".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_parsing_with_kebab_case() {
        let toml_str = r#"
            enabled = true
            max-packet-rate = 300.0
            burst-capacity = 100.0
            kick-message = "Stop spamming"
        "#;
        let config: PacketLimiterConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert!((config.max_packet_rate - 300.0).abs() < f64::EPSILON);
        assert!((config.burst_capacity - 100.0).abs() < f64::EPSILON);
        assert_eq!(config.kick_message, "Stop spamming");
    }

    #[test]
    fn toml_parsing_with_snake_case() {
        let toml_str = r#"
            enabled = true
            max_packet_rate = 500.0
            burst_capacity = 500.0
            kick_message = "Kicked for spamming packets"
        "#;
        let config: PacketLimiterConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert!((config.max_packet_rate - 500.0).abs() < f64::EPSILON);
        assert!((config.burst_capacity - 500.0).abs() < f64::EPSILON);
        assert_eq!(config.kick_message, "Kicked for spamming packets");
    }
}
