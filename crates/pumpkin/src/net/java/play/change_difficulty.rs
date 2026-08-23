#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SChangeDifficulty;

impl JavaClient {
    pub async fn handle_change_difficulty(
        &self,
        server: &Server,
        player: &Player,
        packet: &SChangeDifficulty,
    ) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            warn!(
                "Player {} tried to change difficulty without required permissions",
                player.gameprofile.name
            );
            return;
        }

        let current_info = server.level_info.load();
        if current_info.difficulty_locked {
            warn!(
                "Player {} tried to change difficulty while difficulty is locked",
                player.gameprofile.name
            );
            return;
        }

        server.set_difficulty(packet.difficulty, false).await;

        info!(
            "Player {} changed difficulty to {:?}",
            player.gameprofile.name, packet.difficulty
        );
    }
}
