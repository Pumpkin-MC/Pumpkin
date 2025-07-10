use super::Goal;
use crate::entity::mob::MobEntity;
use async_trait::async_trait;
use rand::Rng;
use std::sync::Weak;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[allow(dead_code)]
pub struct AmbientStandGoal {
    mob: Weak<MobEntity>,
    cooldown: AtomicI32,
}

impl AmbientStandGoal {
    #[must_use]
    pub fn new(mob: Weak<MobEntity>) -> Self {
        let entity = Self {
            mob,
            cooldown: AtomicI32::new(0),
        };
        entity.reset_cooldown();

        entity
    }

    fn reset_cooldown(&self) {
        // TODO: should be: this.cooldown = -entity.getMinAmbientStandDelay();
        // TODO: implement when Horses are implemented
        self.cooldown.store(0, Relaxed);
    }
}

#[async_trait]
impl Goal for AmbientStandGoal {
    async fn can_start(&self, mob: &MobEntity) -> bool {
        let cooldown = self.cooldown.fetch_add(1, Relaxed) + 1;
        if cooldown > 0 && mob.get_random().random_range(0..1000) < cooldown {
            self.reset_cooldown();
        }

        false
    }
    async fn should_continue(&self, _mob: &MobEntity) -> bool {
        false
    }

    async fn start(&self, _mob: &MobEntity) {}

    async fn stop(&self, _mob: &MobEntity) {}

    async fn tick(&self, _mob: &MobEntity) {}
}
