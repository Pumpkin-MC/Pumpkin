#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SLockDifficulty;

impl JavaClient {
    pub fn handle_lock_difficulty(
        &self,
        server: &Server,
        player: &Player,
        packet: &SLockDifficulty,
    ) {
        if player.permission_lvl.load() >= PermissionLvl::Two {
            server.set_difficulty_locked(packet.locked);
            info!(
                "Player {} locked difficulty: {}",
                player.gameprofile.name, packet.locked
            );
        } else {
            warn!(
                "Player {} tried to lock difficulty without required permissions",
                player.gameprofile.name
            );
        }
    }
}
