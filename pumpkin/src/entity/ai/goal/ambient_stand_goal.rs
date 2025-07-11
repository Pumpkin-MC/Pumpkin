use super::Goal;
use crate::entity::mob::Mob;
use async_trait::async_trait;
use rand::Rng;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[allow(dead_code)]
pub struct AmbientStandGoal {
    cooldown: AtomicI32,
}

impl Default for AmbientStandGoal {
    fn default() -> Self {
        let entity = Self {
            cooldown: AtomicI32::new(0),
        };
        entity.reset_cooldown();

        entity
    }
}

impl AmbientStandGoal {
    fn reset_cooldown(&self) {
        // TODO: should be: this.cooldown = -entity.getMinAmbientStandDelay();
        // TODO: implement when Horses are implemented
        self.cooldown.store(0, Relaxed);
    }
}

#[async_trait]
impl Goal for AmbientStandGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        let cooldown = self.cooldown.fetch_add(1, Relaxed) + 1;
        if cooldown > 0 && mob.get_random().random_range(0..1000) < cooldown {
            self.reset_cooldown();
        }

        false
    }
    async fn should_continue(&self, _mob: &dyn Mob) -> bool {
        false
    }

    async fn start(&self, _mob: &dyn Mob) {}

    async fn stop(&self, _mob: &dyn Mob) {}

    async fn tick(&self, _mob: &dyn Mob) {}
}
