use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::witch::WitchEntity;
use crate::entity::{EntityBase, mob::MobEntity};

const REACH: f64 = 500.0;

/// Vanilla: `NearestHealableRaiderTargetGoal<Raider>` as wired by `Witch.registerGoals`.
///
/// (`target = Raider.class`, `mustSee = true`, subselector `hasActiveRaid() &&
/// !target.is(EntityTypes.WITCH)`). Finds the nearest hurt raid-mate (excluding other witches)
/// while an active raid is running; `Witch::mob_tick` drives the 200-tick cooldown externally.
pub struct NearestHealableRaiderTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
}

impl NearestHealableRaiderTargetGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            track_target_goal: TrackTargetGoal::with_default(true),
            target: None,
        }
    }

    fn find_target(mob_entity: &MobEntity) -> Option<Arc<dyn EntityBase>> {
        let world = mob_entity.living_entity.entity.world.load();
        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;
        let self_id = mob_entity.living_entity.entity.entity_id;

        let mut candidates: Vec<(Arc<dyn EntityBase>, f64)> = world
            .get_nearby_entities(search_pos, REACH)
            .into_values()
            .filter_map(|entity| {
                if entity.get_entity().entity_id == self_id {
                    return None;
                }
                if entity.get_entity().entity_type == &pumpkin_data::entity::EntityType::WITCH {
                    return None;
                }
                if !entity
                    .get_entity()
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
                {
                    return None;
                }
                let living = entity.get_living_entity()?;
                if !living.entity.is_alive() {
                    return None;
                }
                if living.health.load()
                    >= living.get_attribute_value(&Attributes::MAX_HEALTH) as f32
                {
                    return None;
                }
                let dist = entity
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&search_pos);
                Some((entity, dist))
            })
            .collect();

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.into_iter().map(|(e, _)| e).next()
    }
}

impl Default for NearestHealableRaiderTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for NearestHealableRaiderTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(witch) = mob.cast_any().downcast_ref::<WitchEntity>() else {
                return false;
            };
            if witch.heal_cooldown.load(Relaxed) > 0 {
                return false;
            }
            if !rand::rng().random_bool(0.5) {
                return false;
            }
            if !mob.get_mob_entity().living_entity.has_active_raid() {
                return false;
            }
            self.target = Self::find_target(mob.get_mob_entity());
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(witch) = mob.cast_any().downcast_ref::<WitchEntity>() {
                witch.heal_cooldown.store(to_goal_ticks(200), Relaxed);
            }
            mob.set_mob_target(self.target.clone()).await;
            self.track_target_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
            self.track_target_goal.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
