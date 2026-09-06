#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::game_rules::GameRule;
use pumpkin_protocol::java::client::play::CGameRuleValues;

impl JavaClient {
    pub fn handle_client_status(&self, player: &Arc<Player>, client_status: &SClientCommand) {
        player.update_last_action_time();
        match client_status.action_id.0 {
            SClientCommand::PERFORM_RESPAWN => {
                // Perform respawn
                if player.living_entity.health.load() > 0.0 {
                    return;
                }
                let player_c = player.clone();
                let Some(server) = player.world().server.upgrade() else {
                    return;
                };
                server.spawn_task(async move {
                    player_c
                        .world()
                        .clone()
                        .respawn_player(&player_c, false)
                        .await;

                    {
                        let screen_handler = player_c
                            .current_screen_handler
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let mut screen_handler = screen_handler
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        screen_handler.sync_state();
                    };

                    // Restore abilities based on gamemode after respawn
                    {
                        let mut abilities = player_c
                            .abilities
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        abilities.set_for_gamemode(player_c.gamemode.load());
                    };
                    player_c.send_abilities_update();
                });
            }
            SClientCommand::REQUEST_STATS => {
                // Request stats
                player.send_stats();
            }
            SClientCommand::REQUEST_GAMERULE_VALUES => {
                self.send_game_rule_values(player);
            }
            _ => {
                self.try_kick(&TextComponent::text("Invalid client status"));
            }
        }
    }

    pub fn send_game_rule_values(&self, player: &Player) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            warn!(
                "Player {} tried to request game rule values without required permissions",
                player.gameprofile.name
            );
            return;
        }

        let level_info = player.world().level_info.load();
        let rules: Vec<(String, String)> = GameRule::all()
            .iter()
            .map(|rule| {
                (
                    rule.to_string(),
                    level_info.game_rules.get(rule).to_string(),
                )
            })
            .collect();
        let rules_ref: Vec<(&str, &str)> = rules
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.try_send_packet(&CGameRuleValues::new(&rules_ref));
    }
}
