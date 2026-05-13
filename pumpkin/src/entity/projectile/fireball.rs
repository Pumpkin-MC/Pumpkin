use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

const EXPLOSION_POWER: f32 = 1.0;

pub struct FireballEntity {
    pub thrown: ThrownItemEntity,
    pub explosion_power: f32,
}

impl FireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
        };

        Self {
            thrown,
            explosion_power: EXPLOSION_POWER,
        }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter);
        Self {
            thrown,
            explosion_power: EXPLOSION_POWER,
        }
    }
}

impl NBTStorage for FireballEntity {}

impl EntityBase for FireballEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.thrown.process_tick(caller, server).await })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();

            // Handle entity/block hit
            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let entity_clone = entity.clone();

                tokio::spawn(async move {
                    // Fireball does 6.0 damage in vanilla
                    entity_clone.get_entity().set_on_fire_for(5.0);
                    let _ = entity_clone
                        .damage(
                            entity_clone.as_ref(),
                            6.0,
                            pumpkin_data::damage::DamageType::FIREBALL,
                        )
                        .await;
                });
            }

            let hit_pos = hit.hit_pos();
            // Explosion sets fire if mob griefing is enabled (assuming true for now)
            world.explode(hit_pos, self.explosion_power).await;
        })
    }
}
