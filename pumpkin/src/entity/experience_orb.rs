use std::sync::{Arc, atomic::AtomicU32};

use async_trait::async_trait;
use pumpkin_data::{damage::DamageType, entity::EntityType};
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{server::Server, world::World};

use super::{Entity, EntityBase, living::LivingEntity, player::Player};

pub struct ExperienceOrbEntity {
    entity: Entity,
    amount: u32,
    orb_age: AtomicU32,
}

impl ExperienceOrbEntity {
    pub fn new(entity: Entity, amount: u32) -> Self {
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            amount,
            orb_age: AtomicU32::new(0),
        }
    }

    pub async fn spawn(world: &Arc<World>, position: Vector3<f64>, amount: u32) {
        let mut amount = amount;
        while amount > 0 {
            let i = Self::round_to_orb_size(amount);
            amount -= i;
            let entity = Entity::new(
                Uuid::new_v4(),
                world.clone(),
                position,
                EntityType::EXPERIENCE_ORB,
                false,
            );
            let orb = Arc::new(Self::new(entity, i));
            world.spawn_entity(orb).await;
        }
    }

    fn round_to_orb_size(value: u32) -> u32 {
        if value >= 2477 {
            2477
        } else if value >= 1237 {
            1237
        } else if value >= 617 {
            617
        } else if value >= 307 {
            307
        } else if value >= 149 {
            149
        } else if value >= 73 {
            73
        } else if value >= 37 {
            37
        } else if value >= 17 {
            17
        } else if value >= 7 {
            7
        } else if value >= 3 {
            3
        } else {
            1
        }
    }
}

#[async_trait]
impl EntityBase for ExperienceOrbEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        self.entity.tick(caller, server).await;

        let age = self
            .orb_age
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if age >= 6000 {
            self.entity.remove().await;
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    async fn on_player_collision(&self, player: &Arc<Player>) {
        if player.living_entity.health.load() > 0.0 {
            let mut delay = player.experience_pick_up_delay.lock().await;
            if *delay == 0 {
                *delay = 2;
                player.living_entity.pickup(&self.entity, 1).await;
                player.add_experience_points(self.amount as i32).await;
                // TODO: pickingCount for merging
                self.entity.remove().await;
            }
        }
    }

    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
}
