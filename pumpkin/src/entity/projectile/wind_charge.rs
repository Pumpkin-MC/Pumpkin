use pumpkin_util::math::vector3::Vector3;
use std::{
    f64,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage,
        living::LivingEntity,
        projectile::{ProjectileHit, ThrownItemEntity},
        projectile_deflection::ProjectileDeflectionType,
    },
    server::Server,
};

// WindCharge.java RADIUS = 1.2F.
const EXPLOSION_POWER: f32 = 1.2;
// BreezeWindCharge.java RADIUS = 3.0F.
const BREEZE_EXPLOSION_POWER: f32 = 3.0;
const DEFAULT_DEFLECT_COOLDOWN: u8 = 5;
pub const WIND_CHARGE_GRAVITY: f64 = 0.0;

/// A kind to differentiate both types of wind charges from each other.
enum WindChargeKind {
    /// Represents a wind charge spawned by a player or dispenser.
    /// This wind charge also has a deflect cooldown counter.
    Normal { deflect_cooldown: AtomicU8 },
    /// Represents a wind charge spawned by a breeze.
    Breeze,
}

pub struct WindChargeEntity {
    kind: WindChargeKind,
    thrown_item_entity: ThrownItemEntity,
}

impl WindChargeEntity {
    #[must_use]
    pub const fn new_normal(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Normal {
                deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            },
            thrown_item_entity,
        }
    }

    #[must_use]
    pub const fn new_breeze(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Breeze,
            thrown_item_entity,
        }
    }

    pub const fn deflect_cooldown(&self) -> Option<&AtomicU8> {
        if let WindChargeKind::Normal {
            deflect_cooldown, ..
        } = &self.kind
        {
            Some(deflect_cooldown)
        } else {
            None
        }
    }

    pub async fn create_explosion(&self, position: Vector3<f64>) {
        let power = match self.kind {
            WindChargeKind::Normal { .. } => EXPLOSION_POWER,
            WindChargeKind::Breeze => BREEZE_EXPLOSION_POWER,
        };
        self.get_entity()
            .world
            .load()
            .explode_without_blocks(position, power)
            .await;
    }

    /// Sets this projectile's velocity from a direction vector, power, and spread.
    /// Mirrors `Projectile.spawnProjectileUsingShoot`.
    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        self.thrown_item_entity
            .set_velocity(x, y, z, power, uncertainty);
    }

    pub fn deflect(
        &mut self,
        deflection: &ProjectileDeflectionType,
        deflector: Option<&dyn EntityBase>,
        _from_attack: bool,
    ) -> bool {
        if let Some(cooldown) = self.deflect_cooldown()
            && cooldown.load(Ordering::Relaxed) > 0
        {
            return false;
        }

        deflection.deflect(self, deflector);

        /* TODO: Does this need to be implemented?
        if self.get_entity().world().is_client() {
            self.set_owner();
            self.on_Deflected(from_attack);
        }
         */
        true
    }
}

impl NBTStorage for WindChargeEntity {}

impl EntityBase for WindChargeEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.thrown_item_entity.process_tick(caller, server).await;

            if let Some(cooldown) = self.deflect_cooldown() {
                let cooldown_ticks = cooldown.load(Ordering::Relaxed);
                if cooldown_ticks > 0 {
                    cooldown.store(cooldown_ticks - 1, Ordering::Relaxed);
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.thrown_item_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
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
            self.create_explosion(hit.hit_pos()).await;
        })
    }
}
