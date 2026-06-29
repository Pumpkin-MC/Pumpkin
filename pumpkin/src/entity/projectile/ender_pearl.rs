use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, EntityType, NBTStorage,
        mob::endermite::EndermiteEntity, projectile::ThrownItemEntity,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

const GRAVITY: f64 = 0.03;
const PARTICLE_COUNT: i32 = 32;
const ENDERMITE_SPAWN_CHANCE: f32 = 0.05;

pub struct EnderPearlEntity {
    pub thrown: ThrownItemEntity,
}

impl EnderPearlEntity {
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

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        Self { thrown }
    }
}

impl NBTStorage for EnderPearlEntity {}

impl EntityBase for EnderPearlEntity {
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
            let entity = self.get_entity();
            let world = entity.world.load();

            let attacker = self
                .thrown
                .owner_id
                .and_then(|id| world.get_entity_by_id(id));

            if let (
                ProjectileHit::Entity {
                    entity: hit_entity,
                    hit_pos,
                    ..
                },
                Some(owner),
            ) = (&hit, attacker)
            {
                let victim_ref = &**hit_entity;
                hit_entity
                    .damage_with_context(
                        victim_ref,
                        0.0,
                        DamageType::THROWN,
                        Some(*hit_pos),
                        Some(owner.get_entity()),
                        Some(victim_ref),
                    )
                    .await;
            }

            // Spawn portal particles at hit position
            let hit_pos = hit.hit_pos();
            for _ in 0..PARTICLE_COUNT {
                let offset = Vector3::new(
                    rand::random::<f32>() as f64 - 0.5,
                    rand::random::<f32>() as f64 * 2.0,
                    rand::random::<f32>() as f64 - 0.5,
                );
                let speed = Vector3::new(
                    rand::random::<f64>() - 0.5,
                    0.0,
                    rand::random::<f64>() - 0.5,
                );

                world.spawn_particle(
                    hit_pos.add(&offset),
                    Vector3::new(speed.x as f32, speed.y as f32, speed.z as f32),
                    1.0,
                    1,
                    Particle::Portal,
                );
            }

            if let Some(owner_id) = self.thrown.owner_id
                && let Some(owner) = world.get_entity_by_id(owner_id)
            {
                let should_spawn_endermite = rand::random::<f32>() < ENDERMITE_SPAWN_CHANCE;
                if world.should_spawn_monsters() && should_spawn_endermite {
                    let entity = Entity::new(
                        world.clone(),
                        owner.get_entity().pos.load(),
                        &EntityType::ENDERMITE,
                    );
                    let endermite = EndermiteEntity::new(entity);
                    world.spawn_entity(endermite).await;
                }

                // Teleport position should be position of entity from last tick (tick before collision)
                let teleport_pos = entity.last_pos.load();

                // In vanilla, teleport handles everything including sound
                owner
                    .clone()
                    .teleport(
                        teleport_pos,
                        Some(owner.get_entity().yaw.load()),
                        Some(owner.get_entity().pitch.load()),
                        world.clone(),
                    )
                    .await;

                // Play teleport sound at new position
                world.play_sound(
                    Sound::EntityPlayerTeleport,
                    SoundCategory::Players,
                    &teleport_pos,
                );

                // Deal 5 damage to owner
                owner
                    .damage(
                        owner.as_ref(),
                        5.0,
                        pumpkin_data::damage::DamageType::ENDER_PEARL,
                    )
                    .await;
            }

            world.send_entity_status(entity, EntityStatus::Death);
        })
    }
}
