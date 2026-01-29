use std::sync::Arc;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
};
use crate::entity::projectile::ProjectileHit;
use pumpkin_data::particle::Particle;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_data::entity::{
    EntityType, EntityStatus,
};
use pumpkin_data::damage::DamageType;

pub struct SnowballEntity {
    pub thrown: ThrownItemEntity,
}

impl SnowballEntity {
    pub async fn new(entity: Entity) -> Self {
        // Keep the velocity initialization
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0)).await;

        // Initialize without owner
        let thrown = ThrownItemEntity { entity, owner_id: None, collides_with_projectiles: false };

        Self { thrown }
    }

    pub async fn new_shot(entity: Entity, _shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, _shooter);
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0)).await;
        Self { thrown }
    }
}

impl NBTStorage for SnowballEntity {}

impl EntityBase for SnowballEntity {
    fn tick<'a>(&'a self, caller: Arc<dyn EntityBase>, server: &'a Server) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { 
            self.thrown.process_tick(caller, server).await 
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn is_projectile(&self) -> bool { true }

    fn on_hit<'a>(&'a self, hit: crate::entity::projectile::ProjectileHit) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            
            // Always send particle status regardless of what was hit
            world.send_entity_status(
                self.get_entity(), 
                EntityStatus::PlayDeathSoundOrAddProjectileHitParticles
            ).await;

            // Handle entity-specific damage
            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let entity_clone = entity.clone();
                let self_entity = self.get_entity().clone();
                
                tokio::spawn(async move {
                    let is_blaze = entity_clone.get_entity().entity_type.id == EntityType::BLAZE.id;
                    let damage = if is_blaze { 3.0 } else { 0.0 }; // Only damage blazes

                    entity_clone.damage(entity_clone.as_ref(), damage, DamageType::THROWN).await;
                });
            }
        })
    }
}
