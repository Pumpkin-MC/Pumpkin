use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use rand::RngExt;

/// A goal that makes the mob wander randomly when idle.
///
/// The mob picks a random nearby position and walks to it.
/// This is the standard idle movement behavior for most overworld mobs.
pub struct WanderAroundGoal {
    goal_control: Controls,
    speed: f64,
    target_x: f64,
    target_y: f64,
    target_z: f64,
    chance: i32,
}

impl WanderAroundGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            speed,
            target_x: 0.0,
            target_y: 0.0,
            target_z: 0.0,
            chance: 120,
        })
    }

    #[must_use]
    pub fn with_chance(speed: f64, chance: i32) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            speed,
            target_x: 0.0,
            target_y: 0.0,
            target_z: 0.0,
            chance,
        })
    }

    fn find_target(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        // Pick a random offset within 10 blocks horizontally
        let dx = rng.random_range(-8.0f64..8.0);
        let dz = rng.random_range(-8.0f64..8.0);

        self.target_x = pos.x + dx;
        self.target_y = pos.y;
        self.target_z = pos.z + dz;
        true
    }
}

impl Goal for WanderAroundGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mut rng = mob.get_random();
            if rng.random_range(0..self.chance) != 0 {
                return false;
            }

            self.find_target(mob)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let current_pos = mob_entity.living_entity.entity.pos.load();
            let mut navigator = mob_entity.navigator.lock().await;
            navigator.set_progress(NavigatorGoal {
                current_progress: current_pos,
                destination: pumpkin_util::math::vector3::Vector3::new(
                    self.target_x,
                    self.target_y,
                    self.target_z,
                ),
                speed: self.speed,
            });
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
