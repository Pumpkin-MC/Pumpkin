use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// A goal that makes the mob flee from a specific entity type.
///
/// When a matching entity is within the detection range, the mob
/// runs in the opposite direction at increased speed.
pub struct FleeEntityGoal {
    goal_control: Controls,
    flee_from_type: &'static EntityType,
    flee_distance: f32,
    #[expect(dead_code)]
    slow_speed: f64,
    fast_speed: f64,
    fleeing: bool,
}

impl FleeEntityGoal {
    #[must_use]
    pub fn new(
        flee_from_type: &'static EntityType,
        flee_distance: f32,
        slow_speed: f64,
        fast_speed: f64,
    ) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            flee_from_type,
            flee_distance,
            slow_speed,
            fast_speed,
            fleeing: false,
        })
    }

    fn find_flee_direction(mob: &dyn Mob, threat_pos: Vector3<f64>) -> Vector3<f64> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        // Run in the opposite direction from the threat, with some randomness
        let dx = pos.x - threat_pos.x;
        let dz = pos.z - threat_pos.z;
        let jitter_x = rng.random_range(-2.0f64..2.0);
        let jitter_z = rng.random_range(-2.0f64..2.0);

        Vector3::new(pos.x + dx + jitter_x, pos.y, pos.z + dz + jitter_z)
    }
}

impl Goal for FleeEntityGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let pos = mob_entity.living_entity.entity.pos.load();
            let world = mob_entity.living_entity.entity.world.load();

            let closest = world.get_closest_entity(pos, self.flee_distance.into(), Some(&[self.flee_from_type]));

            if let Some(threat) = closest {
                let threat_pos = threat.get_entity().pos.load();
                let target = Self::find_flee_direction(mob, threat_pos);
                let current_pos = mob_entity.living_entity.entity.pos.load();
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: current_pos,
                    destination: target,
                    speed: self.fast_speed,
                });
                self.fleeing = true;
                true
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !self.fleeing {
                return false;
            }
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.fleeing = false;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
