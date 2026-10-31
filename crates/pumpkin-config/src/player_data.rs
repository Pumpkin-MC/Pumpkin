use serde::{Deserialize, Serialize};

/// Configuration for player data persistence.
///
/// Controls whether player data is saved and the save interval.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct PlayerDataConfig {
    /// Whether saving player data is enabled.
    #[toml(default = Self::default().save_player_data)]
    pub save_player_data: bool,
    /// Time interval in seconds between automatic player data saves.
    #[toml(default = Self::default().save_player_cron_interval)]
    pub save_player_cron_interval: u64,
}

impl Default for PlayerDataConfig {
    fn default() -> Self {
        Self {
            save_player_data: true,
            save_player_cron_interval: 300,
        }
    }
}
