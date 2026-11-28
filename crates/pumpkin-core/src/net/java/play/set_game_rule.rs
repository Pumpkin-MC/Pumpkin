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
        for entry in &packet.entries {
            let key = entry
                .game_rule_key
                .strip_prefix("minecraft:")
                .unwrap_or(entry.game_rule_key);
            let Some(rule) = GameRule::all().iter().find(|r| r.to_string() == key) else {
                warn!("Unknown game rule: {}", entry.game_rule_key);
                continue;
            };

            let value = match world.get_game_rule(rule) {
                GameRuleValue::Int(_) => entry.value.parse::<i64>().ok().map(GameRuleValue::Int),
                GameRuleValue::Bool(_) => entry.value.parse::<bool>().ok().map(GameRuleValue::Bool),
            };
            let Some(value) = value else {
                warn!(
                    "Player {} tried to set invalid value '{}' for gamerule {}",
                    player.gameprofile.name, entry.value, key
                );
                continue;
            };
            // `World::set_game_rule` refuses rules this world's features leave out, like vanilla
            // `GameRules.set`.
            if world.set_game_rule(rule, value) {
                // TODO vanilla `broadcastGameRuleChangeToOperators`: send `commands.gamerule.set` to ops.
                info!(
                    "Player {} set gamerule {} to {}",
                    player.gameprofile.name, key, entry.value
                );
            }
        }
    }
}
