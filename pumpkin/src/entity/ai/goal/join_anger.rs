//! Pack anger: adopt the combat target of a nearby ally of the same entity type.
//! Used by zombified piglins (vanilla group-anger stand-in).

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;

const SEARCH_RANGE: f64 = 16.0;

pub struct JoinAngerGoal {
    ally_type: &'static EntityType,
}

impl JoinAngerGoal {
    #[must_use]
    pub fn new(ally_type: &'static EntityType) -> Box<Self> {
        Box::new(Self { ally_type })
    }
}

impl Goal for JoinAngerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            let entity = mob.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let self_id = entity.entity_id;

            for ally in world.get_nearby_entities(pos, SEARCH_RANGE).into_values() {
                if ally.get_entity().entity_id == self_id {
                    continue;
                }
                if ally.get_entity().entity_type.id != self.ally_type.id {
                    continue;
                }
                // Ally must also be a MobEntity (all zombified piglins are).
                // We read anger via last_attacker of ally living, or target if we can cast.
                // Prefer: living entity last_attacker that hurt the ally recently.
                let Some(ally_living) = ally.get_living_entity() else {
                    continue;
                };
                let attacker_id = ally_living
                    .last_attacker_id
                    .load(std::sync::atomic::Ordering::Relaxed);
                if attacker_id == 0 {
                    continue;
                }
                let Some(attacker) = world.get_entity_by_id(attacker_id).or_else(|| {
                    world
                        .get_player_by_id(attacker_id)
                        .map(|p| p as std::sync::Arc<dyn crate::entity::EntityBase>)
                }) else {
                    continue;
                };
                if !attacker.get_entity().is_alive() {
                    continue;
                }
                if attacker.get_entity().entity_id == self_id {
                    continue;
                }
                *mob.get_mob_entity().target.lock().await = Some(attacker);
                return true;
            }
            false
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            mob.get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}
