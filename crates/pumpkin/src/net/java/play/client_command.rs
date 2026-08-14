#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_client_status(&self, player: &Arc<Player>, client_status: SClientCommand) {
        player.update_last_action_time();
        match client_status.action_id.0 {
            0 => {
                // Perform respawn
                if player.living_entity.health.load() > 0.0 {
                    return;
                }
                player.world().clone().respawn_player(player, false).await;

                {
                    let screen_handler = player.current_screen_handler.lock().await;
                    let mut screen_handler = screen_handler.lock().await;
                    screen_handler.sync_state().await;
                };

                // Restore abilities based on gamemode after respawn
                {
                    let mut abilities = player.abilities.lock().await;
                    abilities.set_for_gamemode(player.gamemode.load());
                };
                player.send_abilities_update().await;
            }
            1 => {
                // Request stats
                player.send_stats().await;
            }
            _ => {
                self.kick(TextComponent::text("Invalid client status"))
                    .await;
            }
        }
    }
}
