use super::Goal;
use crate::entity::mob::MobEntity;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use rand::Rng;
use std::sync::Weak;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[allow(dead_code)]
pub struct LookAroundGoal {
    mob: Weak<MobEntity>,
    delta_x: AtomicCell<f64>,
    delta_z: AtomicCell<f64>,
    look_time: AtomicI32,
}

impl LookAroundGoal {
    #[must_use]
    pub fn new(mob_weak: Weak<MobEntity>) -> Self {
        Self {
            mob: mob_weak,
            delta_x: AtomicCell::new(0.0),
            delta_z: AtomicCell::new(0.0),
            look_time: AtomicI32::new(0),
        }
    }
}

#[async_trait]
impl Goal for LookAroundGoal {
    async fn can_start(&self, mob: &MobEntity) -> bool {
        mob.get_random().random::<f32>() < 0.02
    }

    async fn should_continue(&self, _mob: &MobEntity) -> bool {
        self.look_time.load(Relaxed) >= 0
    }

    async fn start(&self, mob: &MobEntity) {
        let d = std::f64::consts::TAU * mob.get_random().random::<f64>();
        self.delta_x.store(d.cos());
        self.delta_z.store(d.sin());
        let look_time = 20 + mob.get_random().random_range(0..20);
        self.look_time.store(look_time, Relaxed);
    }

    async fn stop(&self, _mob: &MobEntity) {}

    async fn tick(&self, mob: &MobEntity) {
        self.look_time.fetch_sub(1, Relaxed);
        let look_control = mob.look_control.lock().await;
        let pos = mob.living_entity.entity.pos.load();
        look_control.look_at(
            mob,
            pos.x + self.delta_x.load(),
            mob.living_entity.entity.get_eye_y(),
            pos.z + self.delta_z.load(),
        );
    }
}
