use core::f32;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::{entity::EntityBaseFuture, server::Server, world::World};

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity, player::Player};

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
                &EntityType::EXPERIENCE_ORB,
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

impl NBTStorage for ExperienceOrbEntity {}

impl EntityBase for ExperienceOrbEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            entity.tick(caller.clone(), server).await;
            let bounding_box = entity.bounding_box.load();

            let original_velo = entity.velocity.load();

            let mut velo = original_velo;

            let no_clip = !self
                .entity
                .world
                .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7))
                .await;
            // TODO: isSubmergedIn
            if !no_clip {
                velo.y -= self.get_gravity();
            }

            entity.velocity.store(velo);

            entity.move_entity(caller.clone(), velo).await;

            entity.tick_block_collisions(&caller, server).await;

            let age = self.orb_age.fetch_add(1, Ordering::Relaxed);
            if age >= 6000 {
                self.entity.remove().await;
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
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
        })
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        0.03
    }
}
