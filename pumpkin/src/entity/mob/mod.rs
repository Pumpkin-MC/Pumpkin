use core::f32;
use std::sync::Arc;

use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;

use super::{
    Entity, EntityBase,
    ai::{goal::Goal, path::Navigator},
    living::LivingEntity,
};

pub mod zombie;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals: Mutex<Vec<(Arc<dyn Goal>, bool)>>,
    pub navigator: Mutex<Navigator>,
}

#[async_trait]
impl EntityBase for MobEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        self.living_entity.tick(caller, server).await;
        let mut goals = self.goals.lock().await;
        for (goal, running) in goals.iter_mut() {
            if *running {
                if goal.should_continue(self).await {
                    goal.tick(self).await;
                } else {
                    *running = false;
                }
            } else {
                *running = goal.can_start(self).await;
            }
        }
        let mut navigator = self.navigator.lock().await;
        navigator.tick(&self.living_entity).await;
    }

    async fn damage_with_context(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.living_entity
            .damage_with_context(amount, damage_type, position, source, cause)
            .await
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }
}
