#[allow(clippy::wildcard_imports)]
use super::*;
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

use pumpkin_data::game_rules::GameRule;
use pumpkin_protocol::java::client::play::{CGameEvent, CGameRuleValues, GameEvent};

static RESPAWNING_PLAYERS: LazyLock<Mutex<HashSet<uuid::Uuid>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct RespawnClaim(uuid::Uuid);

impl Drop for RespawnClaim {
    fn drop(&mut self) {
        RESPAWNING_PLAYERS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.0);
    }
}

/// Claims a player's respawn flow so duplicate status packets cannot spawn overlapping tasks.
fn try_claim_respawn(player_id: uuid::Uuid) -> Option<RespawnClaim> {
    let inserted = RESPAWNING_PLAYERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(player_id);
    inserted.then_some(RespawnClaim(player_id))
}

/// Forces the mandatory hardcore spectator state without firing a cancellable gamemode event.
fn force_hardcore_spectator(player: &Player) {
    if player.gamemode.load() == GameMode::Spectator {
        return;
    }

    player.gamemode.store(GameMode::Spectator);

    let entity = player.get_entity();
    if entity.is_fall_flying() {
        entity.set_fall_flying(false);
    }
    if entity.is_sneaking() {
        entity.set_sneaking(false);
    }
    entity.on_ground.store(false, Ordering::Relaxed);
    player.living_entity.fall_distance.store(0.0);
    entity.invulnerable.store(true, Ordering::Relaxed);
    entity.no_physics.store(true, Ordering::Relaxed);

    player.world().broadcast_packet_all(&CPlayerInfoUpdate::new(
        PlayerInfoFlags::UPDATE_GAME_MODE.bits(),
        &[pumpkin_protocol::java::client::play::Player {
            uuid: player.gameprofile.id,
            actions: &[PlayerAction::UpdateGameMode(
                (GameMode::Spectator as i32).into(),
            )],
        }],
    ));

    player.client.try_enqueue_packet_editioned(
        &CGameEvent::new(GameEvent::ChangeGameMode, GameMode::Spectator as i32 as f32),
        &pumpkin_protocol::bedrock::client::set_player_gamemode::CSetPlayerGameType {
            player_game_type: GameMode::Spectator.into(),
        },
    );
}

impl JavaClient {
    pub fn handle_client_status(&self, player: &Arc<Player>, client_status: &SClientCommand) {
        player.update_last_action_time();
        match client_status.action_id.0 {
            SClientCommand::PERFORM_RESPAWN => {
                // Perform respawn
                if player.living_entity.health.load() > 0.0 {
                    return;
                }
                let Some(server) = player.world().server.upgrade() else {
                    return;
                };
                let Some(respawn_claim) = try_claim_respawn(player.gameprofile.id) else {
                    return;
                };
                let player_c = player.clone();
                let is_hardcore = server.basic_config.hardcore;
                server.spawn_task(async move {
                    // Hardcore's "Spectate World" ignores the player's bed/anchor and uses the
                    // overworld spawn. Preserve the saved respawn point so spectating does not
                    // mutate persistent player data just to choose this one respawn location.
                    let saved_respawn_point = if is_hardcore {
                        player_c
                            .respawn_point
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .take()
                    } else {
                        None
                    };

                    player_c
                        .world()
                        .clone()
                        .respawn_player(&player_c, false)
                        .await;

                    if is_hardcore {
                        *player_c
                            .respawn_point
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) =
                            saved_respawn_point;
                        force_hardcore_spectator(&player_c);
                    }

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
                    drop(respawn_claim);
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

        let world = player.world();
        let level_info = world.level_info.load();
        let minecart_improvements_enabled = world.server.upgrade().map_or_else(
            || {
                level_info
                    .data_packs
                    .enabled
                    .iter()
                    .any(|p| p == "minecart_improvements" || p == "file/minecart_improvements")
            },
            |s| s.is_feature_enabled("minecraft:minecart_improvements"),
        );

        let rules: Vec<(String, String)> = GameRule::all()
            .iter()
            .filter(|rule| match rule {
                GameRule::MaxMinecartSpeed => minecart_improvements_enabled,
                _ => true,
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn respawn_claim_allows_only_one_in_flight_respawn() {
        let player_id = Uuid::from_u128(1);
        let claim = try_claim_respawn(player_id).expect("first respawn should be claimed");

        assert!(try_claim_respawn(player_id).is_none());

        drop(claim);
        assert!(try_claim_respawn(player_id).is_some());
    }

    #[test]
    fn mandatory_hardcore_spectator_transition_has_internal_api() {
        let _: fn(&Player) = force_hardcore_spectator;
    }
}
