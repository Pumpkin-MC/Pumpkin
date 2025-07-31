use super::{Control, Goal, GoalControl};
use crate::entity::mob::Mob;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use rand::Rng;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[allow(dead_code)]
pub struct LookAroundGoal {
    goal_control: GoalControl,
    delta_x: AtomicCell<f64>,
    delta_z: AtomicCell<f64>,
    look_time: AtomicI32,
}

impl Default for LookAroundGoal {
    fn default() -> Self {
        Self {
            goal_control: GoalControl::from_array(&[Control::Move, Control::Look]),
            delta_x: AtomicCell::new(0.0),
            delta_z: AtomicCell::new(0.0),
            look_time: AtomicI32::new(0),
        }
    }
}

#[async_trait]
impl Goal for LookAroundGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        mob.get_random().random::<f32>() < 0.02
    }

    async fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.look_time.load(Relaxed) >= 0
    }

    async fn start(&self, mob: &dyn Mob) {
        let d = std::f64::consts::TAU * mob.get_random().random::<f64>();
        self.delta_x.store(d.cos());
        self.delta_z.store(d.sin());
        let look_time = 20 + mob.get_random().random_range(0..20);
        self.look_time.store(look_time, Relaxed);
    }

    async fn stop(&self, _mob: &dyn Mob) {}

    async fn tick(&self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        self.look_time.fetch_sub(1, Relaxed);
        let look_control = mob_entity.look_control.lock().await;
        let pos = mob_entity.living_entity.entity.pos.load();
        look_control.look_at(
            mob,
            pos.x + self.delta_x.load(),
            mob_entity.living_entity.entity.get_eye_y(),
            pos.z + self.delta_z.load(),
        );
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}
