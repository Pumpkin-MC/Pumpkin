use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Vanilla `ResetUniversalAngerTargetGoal<T extends Mob & NeutralMob>`.
///
/// While the `universal_anger` game rule is on, being hurt by a player refreshes this mob's
/// anger timer with no specific grudge target, which makes `NeutralMob.isAngryAtAllPlayers`
/// true for the duration and so re-targets *any* nearby player through the mob's
/// `ActiveTargetGoal`/`isAngryAt` predicate rather than just the attacker. Optionally spreads
/// the same refresh to nearby mobs of the same type.
pub struct ResetUniversalAngerTargetGoal {
    alert_others_of_same_type: bool,
    last_hurt_by_player_timestamp: i32,
}

impl ResetUniversalAngerTargetGoal {
    #[must_use]
    pub fn new(alert_others_of_same_type: bool) -> Box<Self> {
        Box::new(Self {
            alert_others_of_same_type,
            last_hurt_by_player_timestamp: 0,
        })
    }
}

impl Goal for ResetUniversalAngerTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let world = mob_entity.living_entity.entity.world.load();
            if !world.level_info.load().game_rules.universal_anger {
                return false;
            }
            if mob.persistent_anger().is_none() {
                return false;
            }

            let living = &mob_entity.living_entity;
            let attacked_time = living.last_attacked_time.load(Relaxed);
            if attacked_time <= self.last_hurt_by_player_timestamp {
                return false;
            }

            let attacker_id = living.last_attacker_id.load(Relaxed);
            if attacker_id == 0 {
                return false;
            }
            let Some(attacker) = world.get_entity_by_id(attacker_id) else {
                return false;
            };
            attacker.get_entity().entity_type == &EntityType::PLAYER
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            self.last_hurt_by_player_timestamp =
                mob_entity.living_entity.last_attacked_time.load(Relaxed);

            let Some(anger) = mob.persistent_anger() else {
                return;
            };
            anger
                .forget_current_target_and_refresh_universal_anger()
                .await;
            // Vanilla `NeutralMob.stopBeingAngry` (called by the refresh above) also does
            // `setTarget(null)`. `PersistentAnger::stop_being_angry` only clears the grudge
            // state, so drop the mob's live target here too -- without it, `TrackTargetGoal`
            // (via `ActiveTargetGoal`) keeps tracking the stale target instead of
            // re-searching and picking up the newly-true "angry at all players" match.
            mob.set_mob_target(None).await;

            if !self.alert_others_of_same_type {
                return;
            }

            let entity = &mob_entity.living_entity.entity;
            let world = entity.world.load();
            let position = entity.pos.load();
            let follow_range = mob_entity
                .living_entity
                .get_attribute_value(&Attributes::FOLLOW_RANGE);
            let entity_type = entity.entity_type;

            for nearby in world
                .get_nearby_entities(position, follow_range)
                .into_values()
            {
                if nearby.get_entity().entity_id == entity.entity_id
                    || nearby.get_entity().entity_type != entity_type
                {
                    continue;
                }
                let Some(nearby_mob) = nearby.get_mob() else {
                    continue;
                };
                let Some(nearby_anger) = nearby_mob.persistent_anger() else {
                    continue;
                };
                nearby_anger
                    .forget_current_target_and_refresh_universal_anger()
                    .await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
