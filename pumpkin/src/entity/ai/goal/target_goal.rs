use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::entity::{ai::path::NavigatorGoal, mob::MobEntity, player::Player};

use super::Goal;

pub struct TargetGoal {
    // TODO: make this an entity
    target: Mutex<Option<Arc<Player>>>,
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
        // TODO: make this an entity
        let mut target = self.target.lock().await;

        // gets the closest entity (currently player)
        *target = mob
            .living_entity
            .entity
            .world
            .read()
            .await
            .get_closest_player(mob.living_entity.entity.pos.load(), self.range)
            .await;
        // we can't use filter, because of async clousrers
        if let Some(player) = target.as_ref() {
            if player.abilities.lock().await.invulnerable {
                *target = None;
            }
        }

        target.is_some()
    }
    async fn should_continue(&self, mob: &MobEntity) -> bool {
        // if an entity is found, lets check so its in range
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
