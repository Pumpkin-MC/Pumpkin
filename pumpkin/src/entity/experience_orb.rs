use core::f32;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::{entity::EntityBaseFuture, server::Server, world::World};

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity, player::Player};

pub struct ExperienceOrbEntity {
    entity: Entity,
    amount: u32,
    orb_age: AtomicU32,
    count: AtomicU32,
}

impl ExperienceOrbEntity {
    pub fn new(entity: Entity, amount: u32) -> Self {
        entity.yaw.store(rand::random::<f32>() * 360.0);
        Self {
            entity,
            amount,
            orb_age: AtomicU32::new(0),
            count: AtomicU32::new(1),
        }
    }

    pub async fn spawn(world: &Arc<World>, position: Vector3<f64>, amount: u32) {
        let mut amount = amount;
        while amount > 0 {
            let i = Self::round_to_orb_size(amount);
            amount -= i;
            let entity = Entity::new(world.clone(), position, &EntityType::EXPERIENCE_ORB);
            let orb = Arc::new(Self::new(entity, i));
            world.spawn_entity(orb).await;
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

    /// Port of vanilla's `scanForMerges`. Merges compatible orbs (same `amount`, entity ID
    /// difference divisible by 40) within `bounding_box.expand_all(0.5)` into `self`.
    async fn scan_for_merges(&self) {
        let bounding_box = self.entity.bounding_box.load().expand_all(0.5);
        let world = self.entity.world.load();

        for other in world.get_entities_at_box(&bounding_box) {
            let Some(other_orb) = other.cast_any().downcast_ref::<Self>() else {
                continue;
            };
            if std::ptr::eq(self, other_orb) || other_orb.entity.is_removed() {
                continue;
            }
            if other_orb.amount != self.amount
                || other_orb
                    .entity
                    .entity_id
                    .wrapping_sub(self.entity.entity_id)
                    % 40
                    != 0
            {
                continue;
            }

            let other_count = other_orb.count.load(Ordering::Relaxed);
            self.count.fetch_add(other_count, Ordering::Relaxed);
            let other_age = other_orb.orb_age.load(Ordering::Relaxed);
            self.orb_age.fetch_min(other_age, Ordering::Relaxed);
            other_orb.entity.remove().await;
        }
    }
}

impl NBTStorage for ExperienceOrbEntity {}

impl EntityBase for ExperienceOrbEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            entity.tick(caller, server).await;

            let age = self.orb_age.fetch_add(1, Ordering::Relaxed);
            if age > 0 && age.is_multiple_of(20) {
                self.scan_for_merges().await;
            }

            let bounding_box = entity.bounding_box.load();

            let original_velo = entity.velocity.load();

            let mut velo = original_velo;

            let world = entity.world.load();
            if let Some(player) = world.get_closest_player(entity.pos.load(), 8.0)
                && !player.is_spectator()
            {
                let player_entity = player.get_entity();
                let target = player_entity.pos.load()
                    + Vector3::new(0.0, player_entity.get_eye_height() / 2.0, 0.0);
                let delta = target - entity.pos.load();
                let distance = delta.length();
                if distance > 1.0e-4 {
                    let power = (1.0 - distance / 8.0).max(0.0);
                    velo += delta.normalize() * (power * power * 0.1);
                }
            }

            let no_clip = !self
                .entity
                .world
                .load()
                .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7));
            // TODO: isSubmergedIn
            if !no_clip {
                velo.y -= self.get_gravity();
            }

            entity.velocity.store(velo);

            entity.move_entity(caller, velo).await;

            entity.tick_block_collisions(caller, server).await;

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
                    player.living_entity.pickup(&self.entity, 1);
                    let remaining = player.apply_mending_from_xp(self.amount as i32).await;
                    if remaining > 0 {
                        player.add_experience_points(remaining).await;
                    }
                    if self.count.fetch_sub(1, Ordering::Relaxed) <= 1 {
                        self.entity.remove().await;
                    }
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

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
