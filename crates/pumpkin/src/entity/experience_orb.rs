use core::f32;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

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

    pub fn spawn(world: &Arc<World>, position: Vector3<f64>, amount: u32) {
        let mut amount = amount;
        while amount > 0 {
            let i = Self::round_to_orb_size(amount);
            amount -= i;
            let entity = Entity::new(world.clone(), position, &EntityType::EXPERIENCE_ORB);
            let orb = Arc::new(Self::new(entity, i));
            world.spawn_entity(orb);
        }
    }

    const fn round_to_orb_size(value: u32) -> u32 {
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

impl EntityBase for ExperienceOrbEntity {
    fn tick(&self, caller: &Arc<dyn EntityBase>, server: &Server) {
        let entity = &self.entity;
        entity.tick(caller, server);
        let bounding_box = entity.bounding_box.load();

        let original_velo = entity.velocity.load();

        let mut velo = original_velo;

        let no_physics = !self
            .entity
            .world
            .load()
            .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7));
        self.entity.no_physics.store(no_physics, Ordering::Relaxed);
        // TODO: isSubmergedIn
        if !no_physics {
            velo.y -= self.get_gravity();
        }

        entity.velocity.store(velo);

        entity.move_entity(caller, velo);

        entity.tick_block_collisions(caller, server);

        let age = self.orb_age.fetch_add(1, Ordering::Relaxed);
        if age >= 6000 {
            entity.remove();
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn on_player_collision(&self, player: &Arc<Player>) {
        if player.living_entity.health.load() > 0.0 {
            let can_pickup = if let Ok(mut delay) = player.experience_pick_up_delay.try_lock()
                && *delay == 0
            {
                *delay = 2;
                true
            } else {
                false
            };
            if can_pickup {
                player.living_entity.pickup(&self.entity, 1);
                self.entity.remove();
                let amount = self.amount as i32;
                let remaining = player.apply_mending_from_xp(amount);
                if remaining > 0 {
                    player.add_experience_points(remaining);
                }
            }
        }
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    fn get_gravity(&self) -> f64 {
        0.03
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
