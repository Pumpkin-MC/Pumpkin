#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_ground(&self, player: &Player, ground: &SSetPlayerGround) {
        player
            .living_entity
            .entity
            .on_ground
            .store(ground.on_ground, Ordering::Relaxed);
    }
}
