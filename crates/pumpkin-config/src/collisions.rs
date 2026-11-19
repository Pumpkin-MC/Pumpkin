use serde::{Deserialize, Serialize};

/// Configuration for collisions between entities.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CollisionsConfig {
    /// Whether players push each other. Off, players walk through each other
    /// whatever team they are in, also teams of plugins, and the clients are told that no
    /// team collides. Players still push mobs, but mobs no longer push Java players.
    pub player_collisions: bool,
}

impl Default for CollisionsConfig {
    fn default() -> Self {
        Self {
            player_collisions: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CollisionsConfig;

    #[test]
    fn players_collide_unless_the_file_says_otherwise() {
        assert!(CollisionsConfig::default().player_collisions);
        let config: crate::AdvancedConfiguration = toml::from_str("").unwrap();
        assert!(config.collisions.player_collisions);
        let config: crate::AdvancedConfiguration =
            toml::from_str("[collisions]\nplayer_collisions = false").unwrap();
        assert!(!config.collisions.player_collisions);
    }
}
