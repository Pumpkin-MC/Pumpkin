use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{cos, sin};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Goal, to_goal_ticks},
    breath::{DROWNING_DAMAGE, DROWNING_INTERVAL, MAX_AIR},
    mob::{Mob, MobEntity},
};

const SQUID_FLEE_SPEED: f64 = 3.0;
const SQUID_FLEE_MIN_DISTANCE: f64 = 5.0;
const SQUID_FLEE_MAX_DISTANCE: f64 = 10.0;

/// The swim direction the goals pick and `ai_step` applies.
type MovementVector = Arc<AtomicCell<Vector3<f64>>>;

/// Vanilla `Squid`, which `GlowSquidEntity` builds on.
pub struct SquidEntity {
    pub mob_entity: MobEntity,
    movement_vector: MovementVector,
    tentacle_movement: AtomicCell<f32>,
    tentacle_speed: AtomicCell<f32>,
    air_supply: AtomicI32,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self::create(entity))
    }

    /// Builds the squid unwrapped so `GlowSquidEntity` can embed it.
    pub(crate) fn create(entity: Entity) -> Self {
        let squid = Self {
            mob_entity: MobEntity::new(entity),
            movement_vector: MovementVector::default(),
            tentacle_movement: AtomicCell::new(0.0),
            tentacle_speed: AtomicCell::new(random_tentacle_speed()),
            air_supply: AtomicI32::new(MAX_AIR),
        };
        squid.register_goals();
        squid
    }

    fn register_goals(&self) {
        let mut goals = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        goals.add_goal(
            0,
            Box::new(SquidRandomMovementGoal {
                movement_vector: self.movement_vector.clone(),
            }),
        );
        goals.add_goal(
            1,
            Box::new(SquidFleeGoal {
                movement_vector: self.movement_vector.clone(),
            }),
        );
    }

    /// The squid's own part of vanilla `Squid.aiStep`, which follows the shared AI and travel.
    fn ai_step(&self) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let mut tentacle_movement = self.tentacle_movement.load() + self.tentacle_speed.load();
        if tentacle_movement > TAU {
            tentacle_movement -= TAU;
            if self.get_random().random_range(0..10) == 0 {
                self.tentacle_speed.store(random_tentacle_speed());
            }
            // The client holds its tentacles still until this restarts the stroke.
            entity
                .world
                .load()
                .send_entity_status(entity, EntityStatus::SquidAnimSynch, None);
        }
        self.tentacle_movement.store(tentacle_movement);

        let mut velocity = entity.velocity.load();
        if entity.touching_water.load(Ordering::Relaxed) {
            if tentacle_movement < PI {
                if tentacle_movement / PI > 0.75 {
                    velocity = self.movement_vector.load();
                }
            } else {
                velocity = velocity * 0.9;
            }

            let body_yaw = entity.body_yaw.load();
            let target_yaw = -(velocity.x.atan2(velocity.z) as f32).to_degrees();
            let body_yaw = body_yaw + (target_yaw - body_yaw) * 0.1;
            entity.body_yaw.store(body_yaw);
            entity.yaw.store(body_yaw);
        } else {
            // Out of water the shared air travel already applied vanilla's gravity and drag.
            if let Some(levitation) = living.get_effect(&StatusEffect::LEVITATION) {
                velocity.y = 0.05 * (f64::from(levitation.amplifier) + 1.0) * f64::from(0.98f32);
            }
            velocity.x = 0.0;
            velocity.z = 0.0;
        }
        entity.velocity.store(velocity);
    }

    /// Vanilla `Squid.travel`, which Pumpkin only hands over in water.
    fn travel(&self, caller: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        entity.move_entity(caller, entity.velocity.load());
        true
    }

    /// Vanilla `AgeableWaterCreature.handleAirSupply`: a squid suffocates out of water.
    fn handle_air_supply(&self, caller: &dyn EntityBase) {
        let living = &self.mob_entity.living_entity;
        if living.health.load() <= 0.0 || living.entity.touching_water.load(Ordering::Relaxed) {
            self.air_supply.store(MAX_AIR, Ordering::Relaxed);
            return;
        }

        if self.air_supply.fetch_sub(1, Ordering::Relaxed) - 1 <= -DROWNING_INTERVAL {
            self.air_supply.store(0, Ordering::Relaxed);
            caller.damage(caller, DROWNING_DAMAGE, DamageType::DROWN);
        }
    }
}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        self.handle_air_supply(caller);
    }

    fn post_tick(&self) {
        self.ai_step();
    }

    fn mob_travel_in_water(&self, caller: &dyn EntityBase) -> bool {
        self.travel(caller)
    }
}

fn random_tentacle_speed() -> f32 {
    1.0 / (rand::random::<f32>() + 1.0) * 0.2
}

fn has_movement_vector(movement_vector: Vector3<f64>) -> bool {
    movement_vector.length_squared() > f64::from(1.0e-5f32)
}

/// Vanilla `getLastHurtByMob`: the living attacker from the last 100 ticks, while it lives.
fn last_hurt_by_mob(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
    let living = &mob.get_mob_entity().living_entity;
    let attacker_id = living.last_attacker_id.load(Ordering::Relaxed);
    let since_attack = living.entity.age.load(Ordering::Relaxed)
        - living.last_attacked_time.load(Ordering::Relaxed);
    if attacker_id == 0 || since_attack > 100 {
        return None;
    }

    let attacker = living.entity.world.load().get_entity_by_id(attacker_id)?;
    let alive = attacker
        .get_living_entity()
        .is_some_and(|attacker| attacker.entity.is_alive() && attacker.health.load() > 0.0);
    alive.then_some(attacker)
}

/// Vanilla `Squid.SquidRandomMovementGoal`.
struct SquidRandomMovementGoal {
    movement_vector: MovementVector,
}

impl Goal for SquidRandomMovementGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        // Vanilla also stops after 100 ticks of `noActionTime`, which Pumpkin doesn't track.
        let mut rng = mob.get_random();
        if rng.random_range(0..to_goal_ticks(50)) == 0
            || !mob.get_entity().touching_water.load(Ordering::Relaxed)
            || !has_movement_vector(self.movement_vector.load())
        {
            let angle = rng.random::<f32>() * TAU;
            self.movement_vector.store(Vector3::new(
                f64::from(cos(angle) * 0.2),
                f64::from(-0.1 + rng.random::<f32>() * 0.2),
                f64::from(sin(angle) * 0.2),
            ));
        }
    }
}

/// Vanilla `Squid.SquidFleeGoal`. Its bubble particle is client only, so it isn't ported.
struct SquidFleeGoal {
    movement_vector: MovementVector,
}

impl Goal for SquidFleeGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        entity.touching_water.load(Ordering::Relaxed)
            && last_hurt_by_mob(mob).is_some_and(|attacker| {
                entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&attacker.get_entity().pos.load())
                    < SQUID_FLEE_MAX_DISTANCE * SQUID_FLEE_MAX_DISTANCE
            })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(attacker) = last_hurt_by_mob(mob) else {
            return;
        };
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let mut away = pos - attacker.get_entity().pos.load();
        let ahead = BlockPos::floored_v(pos + away);
        let world = entity.world.load();
        let ahead_is_air = world.get_block_state(&ahead).is_air();
        if !ahead_is_air
            && !world
                .get_fluid(&ahead)
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
        {
            return;
        }

        let length = away.length();
        if length > 0.0 {
            // Vanilla throws the normalized vector away, so the raw offset gets scaled.
            let mut speed = SQUID_FLEE_SPEED;
            if length > SQUID_FLEE_MIN_DISTANCE {
                speed -= (length - SQUID_FLEE_MIN_DISTANCE)
                    / (SQUID_FLEE_MAX_DISTANCE - SQUID_FLEE_MIN_DISTANCE);
            }
            if speed > 0.0 {
                away = away * speed;
            }
        }
        if ahead_is_air {
            away.y = 0.0;
        }
        self.movement_vector.store(away / 20.0);
    }
}
