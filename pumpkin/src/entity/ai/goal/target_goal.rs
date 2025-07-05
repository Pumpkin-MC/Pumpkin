use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::entity::{ai::path::NavigatorGoal, mob::MobEntity, player::Player, EntityBase};

use super::Goal;

pub struct TargetGoal {
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    range: f64,
}

impl TargetGoal {
    #[must_use]
    pub fn new(range: f64) -> Self {
        Self {
            target: Mutex::new(None),
            range,
        }
    }
}

#[async_trait]
impl Goal for TargetGoal {
    async fn can_start(&self, mob: &MobEntity) -> bool {
        let mut target = self.target.lock().await;

        // gets the closest entity
        *target = mob
            .living_entity
            .entity
            .world
            .read()
            .await
            .get_closest_entity(mob.living_entity.entity.pos.load(), self.range, None)
            .await;
        // we can't use filter because of async closures
        if let Some(entity) = target.as_ref() {
            //if player.abilities.lock().await.invulnerable {
                *target = None;
            //}
        }

        target.is_some()
    }
    async fn should_continue(&self, mob: &MobEntity) -> bool {
        // If an entity is found, let's check if it's in range
        if let Some(target) = self.target.lock().await.as_ref() {
            let mob_pos = mob.living_entity.entity.pos.load();
            let target_pos = target.living_entity.entity.pos.load();
            let abilities = target.abilities.lock().await;
            return !abilities.invulnerable
                && mob_pos.squared_distance_to_vec(target_pos) <= (self.range * self.range);
        }
        false
    }
    async fn tick(&self, mob: &MobEntity) {
        if let Some(target) = self.target.lock().await.as_ref() {
            let mut navigator = mob.navigator.lock().await;
            let target_player = target.living_entity.entity.pos.load();

            navigator.set_progress(NavigatorGoal {
                current_progress: mob.living_entity.entity.pos.load(),
                destination: target_player,
                speed: 0.1,
            });
        }
    }
}
