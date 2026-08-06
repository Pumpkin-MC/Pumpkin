use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use tokio::sync::Mutex;

use pumpkin_data::entity::EntityStatus;
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{EntityBase, mob::Mob};

/// Vanilla: `OfferFlowerGoal.OFFER_TICKS` -- how long the golem holds the offering pose.
const OFFER_TICKS: i32 = 400;
/// Vanilla: `OfferFlowerGoal.OFFER_TARGET_CONTEXT` -- `TargetingConditions.forNonCombat().range(6.0)`.
const OFFER_RANGE: f64 = 6.0;
/// Vanilla rolls `golem.getRandom().nextInt(8000) != 0` every tick the goal isn't running.
const START_CHANCE: i32 = 8000;

/// Makes an iron golem periodically walk up to and face a nearby villager (or copper golem) and
/// hold out a poppy for a while.
///
/// Vanilla source: `net/minecraft/world/entity/ai/goal/OfferFlowerGoal.java`.
///
/// Scope reduction: current decompiled vanilla (`OfferFlowerGoal.stop`) actually *gives* the
/// poppy by inserting it into `CopperGolem.EQUIPMENT_SLOT_ANTENNA` on entities tagged
/// `ACCEPTS_IRON_GOLEM_GIFT` (which in this snapshot is `copper_golem` only -- villagers are
/// merely a valid *target to walk up to and face*, tagged `CANDIDATE_FOR_IRON_GOLEM_GIFT` along
/// with `copper_golem`, but no longer receive an item directly). Pumpkin has no `CopperGolem`
/// entity, so this port keeps the walk/face/400-tick-hold gesture (visible to clients via
/// `EntityStatus::OfferFlower`/`StopOfferFlower`, matching vanilla's entity events 11/34) and
/// skips the antenna-slot item conferral entirely -- there is nothing in Pumpkin that could
/// receive it.
pub struct OfferFlowerGoal {
    goal_control: Controls,
    entity: Mutex<Option<Arc<dyn EntityBase>>>,
    ticks_left: AtomicI32,
}

impl OfferFlowerGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            entity: Mutex::new(None),
            ticks_left: AtomicI32::new(0),
        })
    }

    fn find_candidate(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let self_entity = mob.get_entity();
        let self_uuid = self_entity.entity_uuid;
        let pos = self_entity.pos.load();
        let world = self_entity.world.load();

        let mut best: Option<(Arc<dyn EntityBase>, f64)> = None;
        for (uuid, candidate) in world.get_nearby_entities(pos, OFFER_RANGE) {
            if uuid == self_uuid {
                continue;
            }
            let candidate_type = candidate.get_entity().entity_type;
            if !candidate_type.has_tag(&tag::EntityType::MINECRAFT_CANDIDATE_FOR_IRON_GOLEM_GIFT) {
                continue;
            }
            if !candidate.get_entity().is_alive() {
                continue;
            }

            let dist_sq = candidate
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&pos);
            if dist_sq > OFFER_RANGE * OFFER_RANGE {
                continue;
            }
            if best
                .as_ref()
                .is_none_or(|(_, best_dist)| dist_sq < *best_dist)
            {
                best = Some((candidate, dist_sq));
            }
        }

        best.map(|(entity, _)| entity)
    }
}

impl Default for OfferFlowerGoal {
    fn default() -> Self {
        *Self::new()
    }
}

impl Goal for OfferFlowerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let world = mob.get_entity().world.load();
            // Vanilla: `golem.level().isBrightOutside()`. Pumpkin has no direct equivalent, so
            // approximate with the same day/night tick window used elsewhere (e.g.
            // `VillagerEntity::mob_tick`'s sleep check) for "is it currently night".
            let time_of_day = world.level_time.lock().await.time_of_day;
            let is_night = (12000..=23000).contains(&time_of_day);
            if is_night {
                return false;
            }

            if mob.get_random().random_range(0..START_CHANCE) != 0 {
                return false;
            }

            let candidate = Self::find_candidate(mob);
            let found = candidate.is_some();
            *self.entity.lock().await = candidate;
            found
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.ticks_left.load(Ordering::Relaxed) > 0 })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_left.store(OFFER_TICKS, Ordering::Relaxed);
            let world = mob.get_entity().world.load();
            world.send_entity_status(mob.get_entity(), EntityStatus::OfferFlower);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let world = mob.get_entity().world.load();
            world.send_entity_status(mob.get_entity(), EntityStatus::StopOfferFlower);
            *self.entity.lock().await = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(entity) = self.entity.lock().await.as_ref() {
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_entity_with_range(entity, 30.0, 30.0);
            }
            self.ticks_left.fetch_sub(1, Ordering::Relaxed);
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
