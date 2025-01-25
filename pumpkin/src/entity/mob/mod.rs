use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_protocol::{client::play::CSpawnEntity, codec::var_int::VarInt};
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
    let entity = server.add_mob_entity(entity_type, position, world).await;
    #[expect(clippy::single_match)]
    match entity_type {
        EntityType::Zombie => Zombie::make(&entity.0).await,
        // TODO
        _ => (),
    }
    entity
}

impl MobEntity {
    pub async fn goal<T: Goal + 'static>(&self, goal: T) {
        self.goals.lock().await.push((Arc::new(goal), false));
    }

    pub fn create_spawn_entity_packet(&self, uuid: Uuid) -> CSpawnEntity {
        let e = &self.living_entity.entity;
        let entity_loc = e.pos.load();
        let entity_vel = e.velocity.load();
        CSpawnEntity::new(
            VarInt(e.entity_id),
            uuid,
            VarInt((e.entity_type) as i32),
            entity_loc.x,
            entity_loc.y,
            entity_loc.z,
            e.pitch.load(),
            e.yaw.load(),
            e.yaw.load(), // todo: head_yaw and yaw are swapped, find out why
            0.into(),
            entity_vel.x as f32,
            entity_vel.y as f32,
            entity_vel.z as f32,
        )
    }
}
