use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
};

/// `LlamaSpit.getDefaultGravity` (`LlamaSpit.java:37-40`).
const GRAVITY: f64 = 0.06;

/// A llama's ranged spit attack. `LlamaSpit.java`.
pub struct LlamaSpitEntity {
    pub thrown: ThrownItemEntity,
}

impl LlamaSpitEntity {
    /// Bare constructor for generic entity-type lookup (`entity::type::from_type`), with no
    /// owner and a resting velocity -- mirrors `SnowballEntity::new`.
    pub fn new(entity: Entity) -> Self {
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };
        Self { thrown }
    }

    /// `LlamaSpit(Level, Llama)` (`LlamaSpit.java:29-35`). Vanilla spawns the spit offset to the
    /// shooter's side (using body yaw and bounding-box width) rather than at the shooter's eye
    /// position; `ThrownItemEntity::new` only supports the latter (eye-position) placement, so the
    /// side offset is approximated here as a documented simplification -- the projectile still
    /// spawns at roughly llama-head height and travels correctly, it just doesn't visually leave
    /// from beside the mouth.
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self { thrown }
    }
}

impl NBTStorage for LlamaSpitEntity {}

impl EntityBase for LlamaSpitEntity {
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

    /// `LlamaSpit.onHitEntity` (`LlamaSpit.java:65-72`): 1 damage from the `spit` damage source,
    /// approximated here with `DamageType::THROWN` (the same source `SnowballEntity` uses for its
    /// blaze-only hit), since Pumpkin has no dedicated "spit" damage type.
    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            world.send_entity_status(self.get_entity(), EntityStatus::Death);

            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let entity_clone = entity.clone();
                tokio::spawn(async move {
                    entity_clone
                        .damage(entity_clone.as_ref(), 1.0, DamageType::THROWN)
                        .await;
                });
            }
        })
    }
}
