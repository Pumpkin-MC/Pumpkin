use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::{
    Controls, Goal, GoalFuture,
    fish_helpers::{find_random_water_target, has_reached_target, is_in_danger, set_move_target},
};
use crate::entity::mob::Mob;

const PANIC_SPEED: f64 = 0.125;

pub struct FishPanicGoal {
    goal_control: Controls,
    target: Option<Vector3<f64>>,
}

impl FishPanicGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            target: None,
        }
    }

    async fn look_for_water(mob: &dyn Mob, xz_dist: i32) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        let base = entity.pos.load();

        let mut best: Option<(f64, Vector3<f64>)> = None;
        for dx in -xz_dist..=xz_dist {
            for dz in -xz_dist..=xz_dist {
                for dy in -1..=1 {
                    let candidate = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    if !world
                        .get_fluid(&candidate)
                        .await
                        .has_tag(&tag::Fluid::MINECRAFT_WATER)
                    {
                        continue;
                    }

                    let target = Vector3::new(
                        f64::from(candidate.0.x) + 0.5,
                        f64::from(candidate.0.y) + 0.5,
                        f64::from(candidate.0.z) + 0.5,
                    );
                    let dist_sq = base.squared_distance_to_vec(&target);
                    match best {
                        Some((best_dist_sq, _)) if dist_sq >= best_dist_sq => {}
                        _ => best = Some((dist_sq, target)),
                    }
                }
            }
        }

        best.map(|(_, target)| target)
    }
}

impl Default for FishPanicGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for FishPanicGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !is_in_danger(mob) {
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            if entity.fire_ticks.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                self.target = Self::look_for_water(mob, 5).await;
            }

            if self.target.is_none() {
                self.target = find_random_water_target(mob, 5.0, 4, 10).await;
            }

            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return false;
            };
            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            !has_reached_target(position, target)
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return;
            };
            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            if has_reached_target(position, target) {
                self.target = None;
                return;
            }
            set_move_target(mob, target, PANIC_SPEED).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
