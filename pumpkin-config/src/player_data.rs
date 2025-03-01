use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerDataConfig {
    /// Is Player Data saving enabled?
    pub save_player_data: bool,
    /// Is Player Data should be cached?
    pub cache_player_data: bool,
    /// Maximum amount of players to cache.
    pub max_cache_entries: u16,
}

impl Default for PlayerDataConfig {
    fn default() -> Self {
        Self {
            save_player_data: true,
            cache_player_data: true,
            max_cache_entries: 256,
        }
    }
}
