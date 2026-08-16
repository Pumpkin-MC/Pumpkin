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
            info!(
                "Player {} requested difficulty lock: {}",
                player.gameprofile.name, packet.locked
            );
            let _ = server;
        }
    }
}
