use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture,
        living::LivingEntity,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

pub const LLAMA_SPIT_GRAVITY: f64 = 0.06;

pub struct LlamaSpitEntity {
    pub thrown: ThrownItemEntity,
}

impl LlamaSpitEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: LLAMA_SPIT_GRAVITY,
        };

        Self { thrown }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let owner_pos = shooter.pos.load();
        let body_yaw_rad = f64::from(shooter.body_yaw.load()).to_radians();
        let bb_width = f64::from(shooter.entity_dimension.load().width);
        let offset = f64::midpoint(bb_width, 1.0);
        let x = owner_pos.x - offset * body_yaw_rad.sin();
        let y = owner_pos.y + shooter.get_eye_height() - 0.1;
        let z = owner_pos.z + offset * body_yaw_rad.cos();
        entity.pos.store(Vector3::new(x, y, z));

        let thrown = ThrownItemEntity {
            entity,
            owner_id: Some(shooter.entity_id),
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: LLAMA_SPIT_GRAVITY,
        };

        Self { thrown }
    }
}

impl EntityBase for LlamaSpitEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.get_entity().touching_water.load(Ordering::Relaxed) {
                self.get_entity().remove().await;
                return;
            }
            self.thrown.process_tick(caller, server).await;
        })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if let ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } = hit
            {
                let entity_clone = entity.clone();
                let world = self.get_entity().world.load();
                let owner_id = self.thrown.owner_id;
                let owner = owner_id.and_then(|id| world.get_entity_by_id(id));

                tokio::spawn(async move {
                    let _ = entity_clone
                        .damage_with_context(
                            entity_clone.as_ref(),
                            1.0,
                            DamageType::SPIT,
                            Some(hit_pos),
                            None,
                            owner.as_deref(),
                        )
                        .await;
                });
            }
        })
    }
}
