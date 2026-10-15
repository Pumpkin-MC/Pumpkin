use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use pumpkin_data::damage::DamageType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
};

pub struct SquidEntity {
    pub mob_entity: MobEntity,
    pub movement_vector: AtomicCell<Vector3<f64>>,
    pub tentacle_movement: AtomicCell<f32>,
    pub tentacle_speed: AtomicCell<f32>,
    pub rotate_speed: AtomicCell<f32>,
    pub air_supply: AtomicI32,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mut rng = rand::rng();
        let tentacle_speed = 1.0 / (rng.random::<f32>() + 1.0) * 0.2;
        let squid = Self {
            mob_entity,
            movement_vector: AtomicCell::new(Vector3::default()),
            tentacle_movement: AtomicCell::new(0.0),
            tentacle_speed: AtomicCell::new(tentacle_speed),
            rotate_speed: AtomicCell::new(0.0),
            air_supply: AtomicI32::new(300),
        };
        Arc::new(squid)
    }
}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn custom_travel(&self, caller: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        entity.move_entity(caller, entity.velocity.load());
        true
    }

    fn on_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.silent.load(Ordering::Relaxed) {
            entity.world.load().play_sound_fine(
                Sound::EntitySquidSquirt,
                SoundCategory::Neutral,
                &entity.pos.load(),
                1.0,
                1.0,
            );
        }
        entity.world.load().spawn_particle(
            entity.pos.load() + Vector3::new(0.0, 0.5, 0.0),
            Vector3::new(0.3, 0.3, 0.3),
            0.1,
            20,
            Particle::SquidInk,
        );
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let in_water = entity.touching_water.load(Ordering::Relaxed);

        // Air supply / drowning on land (matching vanilla WaterAnimal / AgeableWaterCreature)
        if in_water {
            self.air_supply.store(300, Ordering::Relaxed);
        } else {
            let air = self.air_supply.fetch_sub(1, Ordering::Relaxed) - 1;
            if air <= -20 {
                self.air_supply.store(0, Ordering::Relaxed);
                caller.damage(caller, 2.0, DamageType::DROWN);
            }
        }

        let mut rng = rand::rng();

        // 1. Random movement vector selection (SquidRandomMovementGoal from vanilla Squid.java)
        let mut movement_vec = self.movement_vector.load();
        if !in_water || movement_vec.length_squared() < 1e-5 || rng.random_range(0..50) == 0 {
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            movement_vec = Vector3::new(
                (angle.cos() * 0.2) as f64,
                (-0.1 + rng.random::<f32>() * 0.2) as f64,
                (angle.sin() * 0.2) as f64,
            );
            self.movement_vector.store(movement_vec);
        }

        // 2. Tentacle thrust cycle (Squid.aiStep from vanilla Squid.java)
        let mut tentacle_movement = self.tentacle_movement.load() + self.tentacle_speed.load();
        if tentacle_movement > std::f32::consts::TAU {
            tentacle_movement -= std::f32::consts::TAU;
            if rng.random_range(0..10) == 0 {
                let speed = 1.0 / (rng.random::<f32>() + 1.0) * 0.2;
                self.tentacle_speed.store(speed);
            }
        }
        self.tentacle_movement.store(tentacle_movement);

        if in_water {
            let mut rotate_speed = self.rotate_speed.load();
            if tentacle_movement < std::f32::consts::PI {
                let tentacle_scale = tentacle_movement / std::f32::consts::PI;
                if tentacle_scale > 0.75 {
                    entity.velocity.store(movement_vec);
                    rotate_speed = 1.0;
                } else {
                    rotate_speed *= 0.8;
                }
            } else {
                entity.velocity.store(entity.velocity.load() * 0.9);
                rotate_speed *= 0.99;
            }
            self.rotate_speed.store(rotate_speed);

            let vel = entity.velocity.load();
            let horizontal_dist = (vel.x * vel.x + vel.z * vel.z).sqrt();
            if horizontal_dist > 1e-4 {
                let target_yaw = (-vel.x.atan2(vel.z).to_degrees()) as f32;
                let cur_yaw = entity.yaw.load();
                let new_yaw = cur_yaw + (target_yaw - cur_yaw) * 0.1;
                entity.yaw.store(new_yaw);
                entity.body_yaw.store(new_yaw);
                entity.head_yaw.store(new_yaw);
            }
            let target_pitch = (-horizontal_dist.atan2(vel.y).to_degrees()) as f32;
            let cur_pitch = entity.pitch.load();
            entity.pitch.store(cur_pitch + (target_pitch - cur_pitch) * 0.1);

            if !entity.silent.load(Ordering::Relaxed) && rng.random_range(0..120) == 0 {
                entity.world.load().play_sound_fine(
                    Sound::EntitySquidAmbient,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    1.0,
                );
            }
        } else {
            let vel = entity.velocity.load();
            entity.velocity.store(Vector3::new(0.0, (vel.y - 0.08) * 0.98, 0.0));
            let cur_pitch = entity.pitch.load();
            entity.pitch.store(cur_pitch + (-90.0 - cur_pitch) * 0.02);
        }
    }
}

impl CustomSound for SquidEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntitySquidDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntitySquidHurt)
    }
}
