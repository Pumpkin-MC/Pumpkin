use std::sync::Arc;

use crossbeam::atomic::AtomicCell;

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{Goal, GoalFuture, squid_flee::SquidFleeGoal},
    mob::{Mob, MobEntity},
};

pub struct SquidEntity {
    pub mob_entity: MobEntity,
    movement_vector: AtomicCell<Vector3<f64>>,
    tentacle_movement: AtomicCell<f64>,
    tentacle_speed: AtomicCell<f64>,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let squid = Self {
            mob_entity,
            movement_vector: AtomicCell::new(Vector3::default()),
            tentacle_movement: AtomicCell::new(0.0),
            tentacle_speed: AtomicCell::new(1.0 / (rand::random::<f64>() + 1.0) * 0.2),
        };
        let mob_arc = Arc::new(squid);
        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // `Squid.registerGoals` (`animal/squid/Squid.java:57-61`) contains exactly these
            // two goals. They intentionally do not claim MOVE: both write the shared
            // movementVector, and the flee goal runs after random movement and overwrites it.
            goal_selector.add_goal(0, Box::new(SquidRandomMovementGoal));
            goal_selector.add_goal(1, SquidFleeGoal::new());
        };

        mob_arc
    }

    #[must_use]
    pub const fn ink_particle(&self) -> Particle {
        Particle::SquidInk
    }

    #[must_use]
    pub const fn squirt_sound(&self) -> Sound {
        Sound::EntitySquidSquirt
    }

    pub fn set_movement_vector(&self, movement: Vector3<f64>) {
        self.movement_vector.store(movement);
    }

    #[must_use]
    pub fn movement_vector(&self) -> Vector3<f64> {
        self.movement_vector.load()
    }
}

impl NBTStorage for SquidEntity {}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn set_movement_vector(&self, movement: Vector3<f64>) {
        self.set_movement_vector(movement);
    }

    fn get_movement_vector(&self) -> Option<Vector3<f64>> {
        Some(self.movement_vector())
    }

    /// `Squid.travel` (`animal/squid/Squid.java:200-203`) moves using the current velocity and
    /// deliberately skips generic `LivingEntity.travel`. `Squid.aiStep` (`:112-164`) supplies
    /// the water jet propulsion and the gravity/drag fallback outside water.
    fn custom_travel<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let mut velocity = entity.velocity.load();

            if entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let movement = self.movement_vector();
                let phase = self.tentacle_movement.load();
                if phase < std::f64::consts::PI {
                    if phase / std::f64::consts::PI > 0.75 {
                        velocity = movement;
                    }
                } else {
                    velocity = velocity * 0.9;
                }
            } else {
                let levitation = self
                    .mob_entity
                    .living_entity
                    .get_effect(&StatusEffect::LEVITATION)
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

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self
                .mob_entity
                .living_entity
                .dead
                .load(std::sync::atomic::Ordering::Relaxed)
            {
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
        })
    }

    /// `Squid.hurtServer`: on a successful hit with a known attacker, spawns an ink cloud and
    /// plays the squirt sound. The exact "behind and below" rotated-cone particle placement
    /// from vanilla's `spawnInk` isn't ported (no body-rotation state is tracked without the
    /// jet-propulsion physics above); particles are emitted at the squid's own position.
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if source.is_none() {
                return;
            }
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos,
                Vector3::new(0.3, 0.3, 0.3),
                0.1,
                30,
                self.ink_particle(),
            );
            world.play_sound(self.squirt_sound(), SoundCategory::Neutral, &pos);
        })
    }
}

/// `Squid.SquidRandomMovementGoal` (`animal/squid/Squid.java:294-317`).
///
/// The movement vector is shared with `SquidFleeGoal`; because neither goal claims a control, the
/// flee goal can replace this vector in the same tick when the squid was recently hurt.
pub struct SquidRandomMovementGoal;

impl Goal for SquidRandomMovementGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(movement) = mob.get_movement_vector() else {
                return;
            };
            if mob.get_random().random_range(0..50) == 0 || movement.length_squared() <= 1.0e-5 {
                let angle = mob.get_random().random_range(0.0..std::f64::consts::TAU);
                mob.set_movement_vector(Vector3::new(
                    angle.cos() * 0.2,
                    -0.1 + mob.get_random().random_range(0.0..0.2),
                    angle.sin() * 0.2,
                ));
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }
}
