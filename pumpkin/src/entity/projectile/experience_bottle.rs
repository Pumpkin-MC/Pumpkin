use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
};
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;

const GRAVITY: f64 = 0.07;

pub struct ExperienceBottleEntity {
    pub thrown: ThrownItemEntity,
}

impl ExperienceBottleEntity {
    pub fn new(entity: Entity) -> Self {
        // Keep the velocity initialization
        entity.set_velocity(pumpkin_util::math::vector3::Vector3::new(0.0, 0.1, 0.0));

        // Initialize without owner
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self { thrown }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown
            .entity
            .set_velocity(pumpkin_util::math::vector3::Vector3::new(0.0, 0.1, 0.0));
        Self { thrown }
    }
}

impl NBTStorage for ExperienceBottleEntity {}

impl EntityBase for ExperienceBottleEntity {
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
            let hit_pos = hit.hit_pos();

            // Vanilla ThrownExperienceBottle#onHit: level event 2002 (potion-break particle,
            // color -13083194 = green) plays regardless of what was hit.
            let block_pos = BlockPos(pumpkin_util::math::vector3::Vector3::new(
                hit_pos.x.floor() as i32,
                hit_pos.y.floor() as i32,
                hit_pos.z.floor() as i32,
            ));
            world.sync_world_event(
                WorldEvent::ParticlesSpellPotionSplash,
                block_pos,
                -13_083_194,
            );

            // Calculate XP amount: 3 + random(0..5) + random(0..5) = 3..11
            let xp_amount = 3 + rand::random::<u32>() % 5 + rand::random::<u32>() % 5;

            // Spawn experience orbs at hit position
            ExperienceOrbEntity::spawn(&world, hit_pos, xp_amount).await;
        })
    }
}
