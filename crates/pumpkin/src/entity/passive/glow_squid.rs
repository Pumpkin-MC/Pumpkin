use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::squid_flee::SquidFleeGoal,
    mob::{Mob, MobEntity},
    passive::squid::SquidRandomMovementGoal,
};

/// `GlowSquid.hurtServer`: dark ticks reset to this value on a successful hit.
const DARK_TICKS_ON_HURT: i32 = 100;

/// Represents a Glow Squid, a passive aquatic mob that emits a glowing particle effect.
///
/// Wiki: <https://minecraft.wiki/w/Glow_Squid>
pub struct GlowSquidEntity {
    pub mob_entity: MobEntity,
    dark_ticks_remaining: AtomicI32,
    movement_vector: crossbeam::atomic::AtomicCell<Vector3<f64>>,
    tentacle_movement: crossbeam::atomic::AtomicCell<f64>,
    tentacle_speed: crossbeam::atomic::AtomicCell<f64>,
}

impl GlowSquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let glow_squid = Self {
            mob_entity,
            dark_ticks_remaining: AtomicI32::new(0),
            movement_vector: crossbeam::atomic::AtomicCell::new(Vector3::default()),
            tentacle_movement: crossbeam::atomic::AtomicCell::new(0.0),
            tentacle_speed: crossbeam::atomic::AtomicCell::new(
                1.0 / (rand::random::<f64>() + 1.0) * 0.2,
            ),
        };
        let mob_arc = Arc::new(glow_squid);
        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Glow squid inherits `Squid.registerGoals`; see `Squid.java:57-61`.
            goal_selector.add_goal(0, Box::new(SquidRandomMovementGoal));
            goal_selector.add_goal(1, SquidFleeGoal::new());
        };

        mob_arc
    }

    fn send_dark_ticks(&self, ticks: i32) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::DARK_TICKS_REMAINING,
                MetaDataType::INT,
                VarInt(ticks),
            )],
            None,
        );
    }
}

impl NBTStorage for GlowSquidEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int(
                "DarkTicksRemaining",
                self.dark_ticks_remaining.load(Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.dark_ticks_remaining
                .store(nbt.get_int("DarkTicksRemaining").unwrap_or(0), Relaxed);
        })
    }
}

impl Mob for GlowSquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn set_movement_vector(&self, movement: Vector3<f64>) {
        self.movement_vector.store(movement);
    }

    fn get_movement_vector(&self) -> Option<Vector3<f64>> {
        Some(self.movement_vector.load())
    }

    fn custom_travel<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let mut velocity = entity.velocity.load();
            if entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let phase = self.tentacle_movement.load();
                if phase < std::f64::consts::PI {
                    if phase / std::f64::consts::PI > 0.75 {
                        velocity = self.movement_vector.load();
                    }
                } else {
                    velocity = velocity * 0.9;
                }
            } else {
                let levitation = self
                    .mob_entity
                    .living_entity
                    .get_effect(&pumpkin_data::effect::StatusEffect::LEVITATION)
                    .await;
                let y = levitation.map_or_else(
                    || velocity.y - self.get_mob_gravity(),
                    |effect| 0.05 * f64::from(effect.amplifier + 1),
                );
                velocity = Vector3::new(0.0, y * 0.98, 0.0);
            }
            entity.set_velocity(velocity);
            entity.move_entity(caller, velocity).await;
            true
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.send_dark_ticks(self.dark_ticks_remaining.load(Relaxed));
        })
    }

    /// `GlowSquid.aiStep`: decrements dark ticks while positive; unconditionally spawns a
    /// `Glow` particle at a random point in the bounding box every tick (dark ticks only gate
    /// the client-side glow-texture dimming, not the particle itself).
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }

            let mut phase = self.tentacle_movement.load() + self.tentacle_speed.load();
            if phase > std::f64::consts::TAU {
                phase -= std::f64::consts::TAU;
                if rand::rng().random_range(0..10) == 0 {
                    self.tentacle_speed
                        .store(1.0 / (rand::random::<f64>() + 1.0) * 0.2);
                }
            }
            self.tentacle_movement.store(phase);

            let remaining = self.dark_ticks_remaining.load(Relaxed);
            if remaining > 0 {
                self.dark_ticks_remaining.store(remaining - 1, Relaxed);
                self.send_dark_ticks(remaining - 1);
            }

            let entity = &self.mob_entity.living_entity.entity;
            let bb = entity.bounding_box.load();
            let mut rng = rand::rng();
            let point = Vector3::new(
                bb.min.x + rng.random_range(0.0..(bb.max.x - bb.min.x)),
                bb.min.y + rng.random_range(0.0..(bb.max.y - bb.min.y)),
                bb.min.z + rng.random_range(0.0..(bb.max.z - bb.min.z)),
            );
            entity.world.load().spawn_particle(
                point,
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::Glow,
            );
        })
    }

    /// `GlowSquid.hurtServer`: on a successful hit, resets dark ticks to 100 (renders the
    /// squid non-glowing for ~5s).
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if source.is_none() {
                return;
            }
            self.dark_ticks_remaining.store(DARK_TICKS_ON_HURT, Relaxed);
            self.send_dark_ticks(DARK_TICKS_ON_HURT);

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos,
                Vector3::new(0.3, 0.3, 0.3),
                0.1,
                30,
                Particle::GlowSquidInk,
            );
            world.play_sound(Sound::EntityGlowSquidSquirt, SoundCategory::Neutral, &pos);
        })
    }
}
