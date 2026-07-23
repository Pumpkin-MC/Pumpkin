//! Iron golem DefendVillageTargetGoal stand-in:
//! if a nearby villager was recently hurt, attack their attacker.

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;
use std::sync::atomic::Ordering::Relaxed;

const SEARCH_RANGE: f64 = 32.0;

pub struct DefendVillagersGoal;

impl DefendVillagersGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Goal for DefendVillagersGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            let entity = mob.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let self_id = entity.entity_id;

            for other in world.get_nearby_entities(pos, SEARCH_RANGE).into_values() {
                if other.get_entity().entity_type.id != EntityType::VILLAGER.id {
                    continue;
                }
                let Some(v_living) = other.get_living_entity() else {
                    continue;
                };
                if !v_living.is_alive() {
                    continue;
                }
                let attacker_id = v_living.last_attacker_id.load(Relaxed);
                if attacker_id == 0 || attacker_id == self_id {
                    continue;
                }
                // Only recent hits (within ~5 seconds).
                let now = v_living.entity.age.load(Relaxed);
                let hit_at = v_living.last_attacked_time.load(Relaxed);
                if now.saturating_sub(hit_at) > 100 {
                    continue;
                }
                let Some(attacker) = world.get_entity_by_id(attacker_id).or_else(|| {
                    world
                        .get_player_by_id(attacker_id)
                        .map(|p| p as std::sync::Arc<dyn EntityBase>)
                }) else {
                    continue;
                };
                if !attacker.get_entity().is_alive() {
                    continue;
                }
                // Don't attack other golems/villagers.
                let aid = attacker.get_entity().entity_type.id;
                if aid == EntityType::IRON_GOLEM.id || aid == EntityType::VILLAGER.id {
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
