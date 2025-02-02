use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use uuid::Uuid;
use zombie::Zombie;

use crate::{server::Server, world::World};

use super::{
    ai::{goal::Goal, path::Navigator},
    living::LivingEntity,
    Entity, EntityBase,
};

pub mod zombie;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals: Mutex<Vec<(Arc<dyn Goal>, bool)>>,
    pub navigator: Mutex<Navigator>,
}

#[async_trait]
impl EntityBase for MobEntity {
    async fn tick(&self) {
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

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn is_invulnerable_to(&self, damage_type: DamageType) -> bool {
        self.get_entity().is_invulnerable_to(damage_type)
    }
}

pub async fn from_type(
    entity_type: EntityType,
    server: &Server,
    position: Vector3<f64>,
    world: &Arc<World>,
) -> (Arc<dyn EntityBase>, Uuid) {
    let (entity, uuid) = server.add_entity(position, entity_type, world);
    let mob = MobEntity {
        living_entity: LivingEntity::new(entity),
        goals: Mutex::new(vec![]),
        navigator: Mutex::new(Navigator::default()),
    };
    #[expect(clippy::single_match)]
    match entity_type {
        EntityType::Zombie => Zombie::make(&mob).await,
        // TODO
        _ => (),
    }
    (Arc::new(mob), uuid)
}

impl MobEntity {
    pub async fn goal<T: Goal + 'static>(&self, goal: T) {
        self.goals.lock().await.push((Arc::new(goal), false));
    }
}
