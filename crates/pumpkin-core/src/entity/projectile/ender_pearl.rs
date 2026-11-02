use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::atomic::AtomicBool;

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{
        Entity, EntityBase, EntityType, mob::endermite::EndermiteEntity,
        projectile::ThrownItemEntity,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityPose, EntityStatus};
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
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
            owner: ProjectileOwner::none(),
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

impl EntityBase for EnderPearlEntity {
    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.thrown.process_tick(caller);
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        let world = entity.world.load();

        let attacker = self.get_owner_entity();

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

        let teleport_pos = entity.last_pos.load();

        // Like vanilla, a zero hit from the pearl, credited to its thrower if any.
        if let ProjectileHit::Entity {
            entity: hit_entity,
            hit_pos,
            ..
        } = &hit
        {
            hit_entity.damage_with_context(
                &**hit_entity,
                0.0,
                DamageType::THROWN,
                Some(*hit_pos),
                Some(self),
                attacker.as_deref(),
            );
        }

        if let Some(owner) = attacker
            && owner.get_entity().is_alive()
            && owner.get_living_entity().is_none_or(|living| {
                living.health.load() > 0.0 && owner.get_entity().pose.load() != EntityPose::Sleeping
            })
        {
            let should_spawn_endermite = rand::random::<f32>() < ENDERMITE_SPAWN_CHANCE;
            if world.should_spawn_monsters() && should_spawn_endermite {
                let entity = Entity::new(
                    world.clone(),
                    owner.get_entity().pos.load(),
                    &EntityType::ENDERMITE,
                );
                let endermite = EndermiteEntity::new(entity);
                world.spawn_entity(endermite);
            }

            // In vanilla, teleport handles everything including sound
            owner.teleport(
                teleport_pos,
                Some(owner.get_entity().yaw.load()),
                Some(owner.get_entity().pitch.load()),
                world.clone(),
            );

            // Play teleport sound at new position
            world.play_sound(
                Sound::EntityPlayerTeleport,
                SoundCategory::Players,
                &teleport_pos,
            );

            // Deal 5 damage to owner
            owner.damage(
                owner.as_ref(),
                5.0,
                pumpkin_data::damage::DamageType::ENDER_PEARL,
            );
        }

        world.send_entity_status(entity, EntityStatus::Death, ActorEventID::Death);
    }
}
