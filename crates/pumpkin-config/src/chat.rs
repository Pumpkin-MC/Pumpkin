use serde::{Deserialize, Serialize};

/// Configuration for in-game chat behaviour.
///
/// Controls chat formatting, display, and anti-spam protection.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ChatConfig {
    /// The custom chat format.
    /// `Note`: it does not apply when secure chat is enabled.
    pub format: String,
    /// Anti-spam protection settings for player chat and commands.
    pub anti_spam: AntiSpamConfig,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            format: "<{DISPLAYNAME}> {MESSAGE}".to_string(),
            anti_spam: AntiSpamConfig::default(),
        }
    }
}

/// Configuration for chat and command anti-spam protection.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AntiSpamConfig {
    /// Whether anti-spam protection is enabled.
    pub enabled: bool,
    /// The spam counter threshold at which a player will be kicked.
    /// Vanilla default is 200 ticks.
    pub spam_threshold: u32,
    /// The amount added to the spam counter for each chat message or command sent.
    /// Vanilla default is 20 ticks.
    pub message_cost: u32,
    /// The amount decayed from the spam counter per server tick.
    /// Vanilla default is 1 tick.
    pub decay_per_tick: u32,
    /// Whether operators/admins bypass the anti-spam check.
    pub ops_bypass: bool,
}

impl Default for AntiSpamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            spam_threshold: 200,
            message_cost: 20,
            decay_per_tick: 1,
            ops_bypass: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chat_config() {
        let config = ChatConfig::default();
        assert_eq!(config.format, "<{DISPLAYNAME}> {MESSAGE}");
        assert!(config.anti_spam.enabled);
        assert_eq!(config.anti_spam.spam_threshold, 200);
        assert_eq!(config.anti_spam.message_cost, 20);
        assert_eq!(config.anti_spam.decay_per_tick, 1);
        assert!(config.anti_spam.ops_bypass);
    }

    #[test]
    fn deserialize_custom_anti_spam() {
        let toml_str = r#"
            format = "<{NAME}> {MESSAGE}"
            [anti_spam]
            enabled = false
            spam_threshold = 100
            message_cost = 10
            decay_per_tick = 2
            ops_bypass = false
        "#;
        let config: ChatConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.format, "<{NAME}> {MESSAGE}");
        assert!(!config.anti_spam.enabled);
        assert_eq!(config.anti_spam.spam_threshold, 100);
        assert_eq!(config.anti_spam.message_cost, 10);
        assert_eq!(config.anti_spam.decay_per_tick, 2);
        assert!(!config.anti_spam.ops_bypass);
    }
}
