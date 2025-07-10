use std::sync::Arc;

use super::{
    Entity, EntityBase,
    ai::{goal::Goal, path::Navigator},
    living::LivingEntity,
};
use crate::entity::ai::control::look_control::LookControl;
use crate::server::Server;
use async_trait::async_trait;
use tokio::sync::Mutex;

pub mod zombie;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub mob: Option<Arc<dyn Mob>>,
    pub goals: Mutex<Vec<(Arc<dyn Goal>, bool)>>,
    pub navigator: Mutex<Navigator>,
    pub target: Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: Mutex<LookControl>,
}

impl MobEntity {
    pub fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }
}

// This trait contains all overridable functions
pub trait Mob: Send + Sync {
    fn get_max_look_yaw_change(&self) -> i32 {
        10
    }

    fn get_max_look_pitch_change(&self) -> i32 {
        40
    }

    fn get_max_head_rotation(&self) -> i32 {
        75
    }
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

        let look_control = self.look_control.lock().await;
        look_control.tick(self).await;
        drop(look_control);
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }
}
