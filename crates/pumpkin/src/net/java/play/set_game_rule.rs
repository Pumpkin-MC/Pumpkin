#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::game_rules::{GameRule, GameRuleValue};
use pumpkin_protocol::java::server::play::SSetGameRule;

impl JavaClient {
    pub fn handle_set_game_rule(&self, player: &Player, packet: &SSetGameRule<'_>) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            warn!(
                "Player {} tried to set game rules without required permissions",
                player.gameprofile.name
            );
            return;
        }

        let world = player.world();
        let minecart_improvements_enabled = world.server.upgrade().map_or_else(
            || {
                world
                    .level_info
                    .load()
                    .data_packs
                    .enabled
                    .iter()
                    .any(|p| p == "minecart_improvements" || p == "file/minecart_improvements")
            },
            |s| s.is_feature_enabled("minecraft:minecart_improvements"),
        );

        for entry in &packet.entries {
            let key = entry
                .game_rule_key
                .strip_prefix("minecraft:")
                .unwrap_or(entry.game_rule_key);
            let Some(rule) = GameRule::all().iter().find(|r| r.to_string() == key) else {
                warn!("Unknown game rule: {}", entry.game_rule_key);
                continue;
            };

            if *rule == GameRule::MaxMinecartSpeed && !minecart_improvements_enabled {
                warn!(
                    "Player {} tried to set game rule {} which is not enabled by feature flags",
                    player.gameprofile.name, entry.game_rule_key
                );
                continue;
            }

            let level_info = player.world().level_info.load();
            let current_val = level_info.game_rules.get(rule);
            match current_val {
                GameRuleValue::Int(_) => {
                    if let Ok(val) = entry.value.parse::<i64>() {
                        player.world().set_game_rule(rule, GameRuleValue::Int(val));
                        info!(
                            "Player {} set gamerule {} to {}",
                            player.gameprofile.name, key, val
                        );
                    } else {
                        warn!(
                            "Player {} tried to set invalid int value '{}' for gamerule {}",
                            player.gameprofile.name, entry.value, key
                        );
                    }
                }
                GameRuleValue::Bool(_) => {
                    if let Ok(val) = entry.value.parse::<bool>() {
                        player.world().set_game_rule(rule, GameRuleValue::Bool(val));
                        info!(
                            "Player {} set gamerule {} to {}",
                            player.gameprofile.name, key, val
                        );
                    } else {
                        warn!(
                            "Player {} tried to set invalid bool value '{}' for gamerule {}",
                            player.gameprofile.name, entry.value, key
                        );
                    }
                }
            }
        }
    }
}
