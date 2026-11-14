use crate::wit::pumpkin::plugin::game_rules::{GameRule, GameRuleValue};
use crate::wit::pumpkin::plugin::world::World;

impl World {
    /// Gets a boolean game rule value from this world. Returns `None` if the rule is an integer rule.
    pub fn get_game_rule_bool(&self, rule: GameRule) -> Option<bool> {
        match self.get_game_rule(rule) {
            GameRuleValue::Bool(v) => Some(v),
            GameRuleValue::Int(_) => None,
        }
    }

    /// Gets an integer game rule value from this world. Returns `None` if the rule is a boolean rule.
    pub fn get_game_rule_int(&self, rule: GameRule) -> Option<i32> {
        match self.get_game_rule(rule) {
            GameRuleValue::Int(v) => Some(v),
            GameRuleValue::Bool(_) => None,
        }
    }

    /// Sets a boolean game rule value for this world.
    pub fn set_game_rule_bool(&self, rule: GameRule, value: bool) {
        self.set_game_rule(rule, GameRuleValue::Bool(value));
    }

    /// Sets an integer game rule value for this world.
    pub fn set_game_rule_int(&self, rule: GameRule, value: i32) {
        self.set_game_rule(rule, GameRuleValue::Int(value));
    }
}
