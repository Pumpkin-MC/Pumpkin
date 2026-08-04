use std::sync::Weak;

use pumpkin_data::Block;
use pumpkin_util::math::position::{BlockPos, BlockPosIterator};

use super::{Controls, Goal, GoalFuture};
use crate::entity::{
    ageable::AgeableMob, ai::pathfinder::NavigatorGoal, mob::Mob, passive::turtle::TurtleEntity,
};

/// `Turtle.TurtleGoToWaterGoal`: babies always seek water; adults only while not already in
/// water, not carrying an egg and not heading home. Recomputes the target every 160 ticks.
const RECALC_INTERVAL: i32 = 160;
const SEARCH_RADIUS: i32 = 8;
const SEARCH_HEIGHT: i32 = 3;

pub struct TurtleGoToWaterGoal {
    turtle: Weak<TurtleEntity>,
    speed: f64,
    target: Option<BlockPos>,
    recalc_cooldown: i32,
}

impl TurtleGoToWaterGoal {
    #[must_use]
    pub fn new(turtle: Weak<TurtleEntity>, speed: f64) -> Box<Self> {
        Box::new(Self {
            turtle,
            speed,
            target: None,
            recalc_cooldown: 0,
        })
    }

    fn find_water(mob: &dyn Mob) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let origin = entity.block_pos.load();
        let world = entity.world.load();

        let mut best: Option<(i32, BlockPos)> = None;
        for pos in BlockPosIterator::new(
            origin.0.x - SEARCH_RADIUS,
            origin.0.y - SEARCH_HEIGHT,
            origin.0.z - SEARCH_RADIUS,
            origin.0.x + SEARCH_RADIUS,
            origin.0.y + SEARCH_HEIGHT,
            origin.0.z + SEARCH_RADIUS,
        ) {
            if world.get_block(&pos) == &Block::WATER {
                let dx = pos.0.x - origin.0.x;
                let dy = pos.0.y - origin.0.y;
                let dz = pos.0.z - origin.0.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if best.is_none_or(|(best_dist, _)| dist_sq < best_dist) {
                    best = Some((dist_sq, pos));
                }
            }
        }
        best.map(|(_, pos)| pos)
    }
}

impl Goal for TurtleGoToWaterGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return false;
            };

            let entity = mob.get_entity();
            if entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return false;
            }
            if !turtle.is_baby() && (turtle.has_egg() || turtle.is_going_home()) {
                return false;
            }

            let Some(target) = Self::find_water(mob) else {
                return false;
            };
            self.target = Some(target);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            !entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.recalc_cooldown = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return;
            };

            if self.recalc_cooldown <= 0 {
                self.recalc_cooldown = RECALC_INTERVAL;
                let my_pos = mob.get_entity().pos.load();
                let dest = target.to_f64();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(my_pos, dest, self.speed));
            } else {
                self.recalc_cooldown -= 1;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
