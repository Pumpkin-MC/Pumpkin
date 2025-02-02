use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::entity::{mob::MobEntity, player::Player};

use super::Goal;

pub struct LookAtEntityGoal {
    // TODO: make this an entity
    target: Mutex<Option<Arc<Player>>>,
    range: f64,
}

impl LookAtEntityGoal {
    #[must_use]
    pub fn new(range: f64) -> Self {
        Self {
            target: Mutex::new(None),
            range,
        }
    }
}

#[async_trait]
impl Goal for LookAtEntityGoal {
    async fn can_start(&self, mob: &crate::entity::mob::MobEntity) -> bool {
        // TODO: make this an entity
        let mut target = self.target.lock().await;

        *target = mob
            .living_entity
            .entity
            .world
            .read()
            .await
            .get_closest_player(mob.living_entity.entity.pos.load(), self.range)
            .await;
        target.is_some()
    }

    async fn should_continue(&self, mob: &MobEntity) -> bool {
        if let Some(target) = self.target.lock().await.as_ref() {
            let mob_pos = mob.living_entity.entity.pos.load();
            let target_pos = target.living_entity.entity.pos.load();
            return mob_pos.squared_distance_to_vec(target_pos) <= (self.range * self.range);
        }
        false
    }

    async fn tick(&self, mob: &MobEntity) {
        if let Some(target) = self.target.lock().await.as_ref() {
            let target_pos = target.living_entity.entity.pos.load();
            mob.living_entity.entity.look_at(target_pos).await;
        }
    }
}
