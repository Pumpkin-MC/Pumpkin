//! Vanilla `MoveThroughVillageGoal`, approximated.
//!
//! Vanilla strolls between village POI sections; Pumpkin has no POI graph
//! yet, so the goal anchors on the claimed bed of a villager within range —
//! at night idle zombies drift toward the village exactly like the original
//! pressure behavior players expect.

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

const VILLAGE_SEARCH_RADIUS: f64 = 48.0;
const ARRIVE_DISTANCE_SQ: f64 = 16.0;
const MAX_RUN_TICKS: i32 = 400;

pub struct MoveThroughVillageGoal {
    speed: f64,
    target: Option<BlockPos>,
    cooldown: i32,
    run_ticks: i32,
    repath_delay: i32,
}

impl MoveThroughVillageGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
            cooldown: 0,
            run_ticks: 0,
            repath_delay: 0,
        }
    }

    async fn is_night(mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        let time = { world.level_time.lock().await.time_of_day.rem_euclid(24000) };
        (13000..23000).contains(&time)
    }

    /// Nearest claimed villager bed — the village anchor.
    fn find_village_anchor(mob: &dyn Mob) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let mut best: Option<(BlockPos, f64)> = None;
        for (_, other) in world.get_nearby_entities(pos, VILLAGE_SEARCH_RADIUS) {
            if other.get_entity().entity_type != &EntityType::VILLAGER {
                continue;
            }
            let Some(home) = other.get_home_pos() else {
                continue;
            };
            let distance = home.to_centered_f64().squared_distance_to_vec(&pos);
            if distance > ARRIVE_DISTANCE_SQ
                && best.is_none_or(|(_, best_distance)| distance < best_distance)
            {
                best = Some((home, distance));
            }
        }
        best.map(|(home, _)| home)
    }
}

impl Goal for MoveThroughVillageGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            // Only idle mobs stroll the village.
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            if !Self::is_night(mob).await {
                return false;
            }
            self.target = Self::find_village_anchor(mob);
            if self.target.is_none() {
                self.cooldown = to_goal_ticks(200);
                return false;
            }
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return false;
            };
            if self.run_ticks > MAX_RUN_TICKS || mob.get_mob_entity().target.lock().await.is_some()
            {
                return false;
            }
            let distance_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.to_centered_f64());
            distance_sq > ARRIVE_DISTANCE_SQ
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.run_ticks = 0;
            self.repath_delay = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.cooldown = to_goal_ticks(200);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.run_ticks += 1;
            self.repath_delay -= 1;
            if self.repath_delay > 0 {
                return;
            }
            self.repath_delay = to_goal_ticks(20);
            let Some(target) = self.target else {
                return;
            };
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(
                mob_pos,
                target.to_centered_f64(),
                self.speed,
            ));
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
