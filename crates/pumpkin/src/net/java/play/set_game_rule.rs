#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SSetGameRule;

impl JavaClient {
    pub fn handle_set_game_rule(&self, player: &Player, packet: &SSetGameRule<'_>) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        info!(
            "Player {} set gamerule {} to {}",
            player.gameprofile.name, packet.rule, packet.value
        );
    }
}
