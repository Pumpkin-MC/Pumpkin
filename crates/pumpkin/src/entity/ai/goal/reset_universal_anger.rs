use std::sync::atomic::Ordering::Relaxed;

use super::Goal;
use crate::entity::ai::util::goal_utils;
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;

/// With `universal_anger` on, a player hit angers the mob at every player.
///
/// Takes the place of the revenge goal, which bows out for players while the gamerule is set.
/// Owns no control: must run beside a running target goal of higher priority.
pub struct ResetUniversalAngerGoal {
    alert_others: bool,
    last_hurt_by_player_time: i32,
}

impl ResetUniversalAngerGoal {
    #[must_use]
    pub fn new(alert_others: bool) -> Box<Self> {
        Box::new(Self {
            alert_others,
            last_hurt_by_player_time: 0,
        })
    }

    fn was_hurt_by_player(&self, mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;
        if living.last_attacked_time.load(Relaxed) <= self.last_hurt_by_player_time {
            return false;
        }
        let attacker_id = living.last_attacker_id.load(Relaxed);
        if attacker_id == 0 {
            return false;
        }
        living
            .entity
            .world
            .load()
            .get_entity_by_id(attacker_id)
            .is_some_and(|attacker| attacker.get_entity().entity_type == &EntityType::PLAYER)
    }
}

impl Goal for ResetUniversalAngerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_entity()
            .world
            .load()
            .level_info
            .load()
            .game_rules
            .universal_anger
            && self.was_hurt_by_player(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.last_hurt_by_player_time = mob
            .get_mob_entity()
            .living_entity
            .last_attacked_time
            .load(Relaxed);

        let Some(neutral) = mob.as_neutral() else {
            return;
        };
        neutral.forget_current_target_and_refresh_universal_anger();

        if !self.alert_others {
            return;
        }
        for other in goal_utils::nearby_same_type(mob) {
            if let Some(other_neutral) = other.get_mob().and_then(Mob::as_neutral) {
                other_neutral.forget_current_target_and_refresh_universal_anger();
            }
        }
    }
}
