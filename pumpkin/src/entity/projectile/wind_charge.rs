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
        Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity,
        projectile::ThrownItemEntity, projectile_deflection::ProjectileDeflectionType,
    },
    server::Server,
};

const EXPLOSION_POWER: f32 = 1.2;
// square(3.5)
const MAX_RENDER_DISTANCE_WHEN_NEWLY_SPAWNED: f32 = 3.5 * 3.5;
const DEFAULT_DEFLECT_COOLDOWN: u8 = 5;

pub struct WindChargeEntity {
    deflect_cooldown: AtomicU8,
    thrown_item_entity: ThrownItemEntity,
}

impl WindChargeEntity {
    #[must_use]
    pub fn new(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            thrown_item_entity,
        }
    }

    pub fn get_deflect_cooldown(&self) -> u8 {
        self.deflect_cooldown.load(Ordering::Relaxed)
    }

    pub fn set_deflect_cooldown(&self, value: u8) {
        self.deflect_cooldown.store(value, Ordering::Relaxed);
    }

    pub async fn create_explosion(&self, position: Vector3<f64>) {
        self.get_entity()
            .world
            .explode(position, EXPLOSION_POWER)
            .await;
    }

    pub fn should_render(&self, distance: f64) -> bool {
        if self.get_entity().age.load(Ordering::Relaxed) < 2
            && distance < f64::from(MAX_RENDER_DISTANCE_WHEN_NEWLY_SPAWNED)
        {
            return false;
        }

        let mut average_side_length = self
            .get_entity()
            .bounding_box
            .load()
            .get_average_side_length();

        if average_side_length.is_nan() {
            average_side_length = 1.0;
        }

        // TODO: IMPLEMENT renderDistanceMultiplier instead of the 1.0
        average_side_length *= 64.0 * 1.0;
        distance < average_side_length * average_side_length
    }

    pub fn deflect(
        &mut self,
        deflection: &ProjectileDeflectionType,
        deflector: Option<&dyn EntityBase>,
        _from_attack: bool,
    ) -> bool {
        if self.deflect_cooldown.load(Ordering::Relaxed) > 0 {
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
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.get_deflect_cooldown() > 0 {
                self.set_deflect_cooldown(self.get_deflect_cooldown() - 1);
            }

            self.thrown_item_entity.tick(caller, server).await;
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
}
