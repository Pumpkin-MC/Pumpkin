use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerDataConfig {
    /// Is Player Data saving enabled?
    pub save_player_data: bool,
    /// Time interval in seconds to save player data
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
