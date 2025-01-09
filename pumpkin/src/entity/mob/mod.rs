use std::sync::Arc;

use pumpkin_entity::entity_type::EntityType;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use uuid::Uuid;
use zombie::Zombie;

use crate::{server::Server, world::World};

use super::{
    ai::{goal::Goal, path::Navigator},
    living::LivingEntity,
};

pub mod zombie;

pub struct MobEntity {
    pub living_entity: Arc<LivingEntity>,
    pub goals: Mutex<Vec<(Arc<dyn Goal>, bool)>>,
    pub navigator: Mutex<Navigator>,
}

impl MobEntity {
    pub async fn tick(&self) {
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
}

pub async fn from_type(
    entity_type: EntityType,
    server: &Server,
    position: Vector3<f64>,
    world: &Arc<World>,
) -> (Arc<MobEntity>, Uuid) {
    match entity_type {
        EntityType::Zombie => Zombie::make(server, position, world).await,
        // TODO
        _ => server.add_mob_entity(entity_type, position, world).await,
    }
}

impl MobEntity {
    pub async fn goal<T: Goal + 'static>(&self, goal: T) {
        self.goals.lock().await.push((Arc::new(goal), false));
    }
}
