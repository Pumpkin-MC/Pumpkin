use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use tokio::sync::Mutex;

use crate::server::Server;

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
    async fn tick(&self, server: &Server) {
        self.living_entity.tick(server).await;
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

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.living_entity.write_nbt(nbt).await;
        // TODO: write mob stuff
    }

    async fn read_nbt(&self, nbt: &NbtCompound) {
        self.living_entity.read_nbt(nbt).await;
        // TODO: read mob stuff
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }
}

impl MobEntity {
    pub async fn goal<T: Goal + 'static>(&self, goal: T) {
        self.goals.lock().await.push((Arc::new(goal), false));
    }
}
