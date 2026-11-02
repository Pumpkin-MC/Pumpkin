use std::sync::{Arc, atomic::Ordering::Relaxed};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    effect::StatusEffect,
    entity::EntityStatus,
    particle::Particle,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::{boundingbox::EntityDimensions, cos, sin, vector3::Vector3};
use rand::RngExt;

use super::ageable_water_creature::AgeableWaterCreature;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::{goal::Goal, pathfinder::node::PathType},
    mob::{Mob, MobEntity},
};

const BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new(0.5, 0.5, 0.37);
const SQUID_FLEE_SPEED: f64 = 3.0;
const SQUID_FLEE_MIN_DISTANCE: f64 = 5.0;
const SQUID_FLEE_MAX_DISTANCE: f64 = 10.0;
const MOVEMENT_VECTOR_THRESHOLD: f64 = 1.0E-5f32 as f64;
const RANDOM_MOVEMENT_INTERVAL: i32 = 50;
const MAX_NO_ACTION_TIME: i32 = 100;
const MOVEMENT_SPEED: f32 = 0.2;
const TENTACLE_SPEED_SCALE: f32 = 0.2;
const VERTICAL_MOVEMENT_OFFSET: f32 = -0.1;
const FLEE_VECTOR_DIVISOR: f64 = 20.0;
pub(super) const SOUND_VOLUME: f32 = 0.4;
const INK_PARTICLE_COUNT: usize = 30;
const INK_DIRECTION_SPREAD: f64 = 0.6;
const INK_PARTICLE_SPEED: f32 = 0.1;
pub(crate) const DEFAULT_GRAVITY: f64 = 0.08;

pub struct SquidData {
    movement_vector: AtomicCell<Vector3<f64>>,
    tentacle_movement: AtomicCell<f32>,
    tentacle_speed: AtomicCell<f32>,
    x_body_rot: AtomicCell<f32>,
    x_body_rot_o: AtomicCell<f32>,
    y_body_rot_o: AtomicCell<f32>,
}

impl SquidData {
    pub(super) fn new(mob: &MobEntity) -> Arc<Self> {
        mob.navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_pathfinding_malus(PathType::Water, 0.0);
        let data = Arc::new(Self {
            movement_vector: AtomicCell::new(Vector3::default()),
            tentacle_movement: AtomicCell::new(0.0),
            tentacle_speed: AtomicCell::new(
                1.0 / (rand::rng().random::<f32>() + 1.0) * TENTACLE_SPEED_SCALE,
            ),
            x_body_rot: AtomicCell::new(0.0),
            x_body_rot_o: AtomicCell::new(0.0),
            y_body_rot_o: AtomicCell::new(0.0),
        });
        mob.goals_selector
            .add_goal(0, Box::new(SquidRandomMovementGoal { data: data.clone() }));
        mob.goals_selector
            .add_goal(1, Box::new(SquidFleeGoal { data: data.clone() }));
        data
    }
}

pub struct SquidEntity {
    pub mob_entity: MobEntity,
    ageable_data: AgeableData,
    squid_data: Arc<SquidData>,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let squid_data = SquidData::new(&mob_entity);
        Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            squid_data,
        })
    }
}

pub trait Squid: AgeableWaterCreature {
    fn squid_data(&self) -> &SquidData;
    fn ambient_sound(&self) -> Sound;
    fn squirt_sound(&self) -> Sound;
    fn ink_particle(&self) -> Particle;

    fn squid_ai_step(&self) {
        let data = self.squid_data();
        let entity = self.get_entity();
        data.x_body_rot_o.store(data.x_body_rot.load());
        let mut tentacle_movement = data.tentacle_movement.load() + data.tentacle_speed.load();
        if f64::from(tentacle_movement) > std::f64::consts::TAU {
            tentacle_movement -= std::f32::consts::TAU;
            if self.get_random().random_range(0..10) == 0 {
                data.tentacle_speed
                    .store(1.0 / (self.get_random().random::<f32>() + 1.0) * TENTACLE_SPEED_SCALE);
            }
            entity.world.load().send_entity_status(
                entity,
                EntityStatus::SquidAnimSynch,
                ActorEventID::None,
            );
        }
        data.tentacle_movement.store(tentacle_movement);
        if entity.is_in_water() {
            if tentacle_movement < std::f32::consts::PI {
                let tentacle_scale = tentacle_movement / std::f32::consts::PI;
                if tentacle_scale > 0.75 {
                    entity.velocity.store(data.movement_vector.load());
                }
            } else {
                entity.velocity.store(entity.velocity.load() * 0.9);
            }
            let movement = entity.velocity.load();
            let body_yaw = entity.body_yaw.load();
            let yaw =
                body_yaw + (-(movement.x.atan2(movement.z) as f32).to_degrees() - body_yaw) * 0.1;
            entity.body_yaw.store(yaw);
            entity.yaw.store(yaw);
            let x_body_rot = data.x_body_rot.load();
            data.x_body_rot.store(
                x_body_rot
                    + (-(movement.x.hypot(movement.z).atan2(movement.y) as f32).to_degrees()
                        - x_body_rot)
                        * 0.1,
            );
        } else {
            let living = &self.get_mob_entity().living_entity;
            let mut yd = entity.velocity.load().y;
            if let Some(effect) = living.get_effect(&StatusEffect::LEVITATION) {
                yd = 0.05 * f64::from(effect.amplifier + 1);
            } else if !entity.has_no_gravity() {
                yd -= self.get_mob_gravity();
            }
            let drag = crate::entity::living::LivingEntity::compute_modified_friction(
                0.98,
                living.get_attribute_value(&pumpkin_data::attributes::Attributes::AIR_DRAG_MODIFIER)
                    as f32,
            );
            entity.velocity.store(Vector3::new(0.0, yd * drag, 0.0));
            let x_body_rot = data.x_body_rot.load();
            data.x_body_rot
                .store(x_body_rot + (-90.0 - x_body_rot) * 0.02);
        }
    }

    fn squid_base_tick(&self, caller: &dyn EntityBase) {
        self.squid_data()
            .y_body_rot_o
            .store(self.get_entity().body_yaw.load());
        self.handle_air_supply(self.get_entity().air_supply.load(Relaxed), caller);
    }

    fn rotate_vector(&self, vector: Vector3<f64>) -> Vector3<f64> {
        let data = self.squid_data();
        let x_rot = data.x_body_rot_o.load().to_radians();
        let y_rot = (-data.y_body_rot_o.load()).to_radians();
        let (sin_x, cos_x) = (sin(x_rot), cos(x_rot));
        let (sin_y, cos_y) = (sin(y_rot), cos(y_rot));
        let y = vector.y * f64::from(cos_x) + vector.z * f64::from(sin_x);
        let z = vector.z * f64::from(cos_x) - vector.y * f64::from(sin_x);
        Vector3::new(
            vector.x * f64::from(cos_y) + z * f64::from(sin_y),
            y,
            z * f64::from(cos_y) - vector.x * f64::from(sin_y),
        )
    }

    fn spawn_ink(&self) {
        self.make_sound(self.squirt_sound());
        let entity = self.get_entity();
        let pos = self.rotate_vector(Vector3::new(0.0, -1.0, 0.0)) + entity.pos.load();
        let mut rng = self.get_random();
        for _ in 0..INK_PARTICLE_COUNT {
            let dir = self.rotate_vector(Vector3::new(
                f64::from(rng.random::<f32>()) * INK_DIRECTION_SPREAD - INK_DIRECTION_SPREAD / 2.0,
                -1.0,
                f64::from(rng.random::<f32>()) * INK_DIRECTION_SPREAD - INK_DIRECTION_SPREAD / 2.0,
            ));
            let ink_pos_offset_scale = if self.is_baby() { 0.1f32 } else { 0.3f32 };
            let offset = dir * f64::from(ink_pos_offset_scale + rng.random::<f32>() * 2.0);
            entity.world.load().spawn_particle(
                pos.add_raw(0.0, 0.5, 0.0),
                Vector3::new(offset.x as f32, offset.y as f32, offset.z as f32),
                INK_PARTICLE_SPEED,
                0,
                self.ink_particle(),
            );
        }
    }

    fn squid_hurt(&self) -> bool {
        if last_hurt_by_mob(self).is_some() {
            self.spawn_ink();
            true
        } else {
            false
        }
    }

    fn update_squid_dimensions(&self) {
        let entity = self.get_entity();
        entity.set_default_dimensions(if self.is_baby() {
            BABY_DIMENSIONS
        } else {
            Entity::type_dimensions(entity.entity_type)
        });
    }

    fn init_squid_data(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::ageable_mob::DATA_BABY_ID,
            self.is_baby(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::ageable_mob::AGE_LOCKED,
            self.is_age_locked(),
        );
        self.update_squid_dimensions();
    }
}

fn last_hurt_by_mob<M: Mob + ?Sized>(mob: &M) -> Option<Arc<dyn EntityBase>> {
    mob.get_mob_entity().living_entity.get_last_hurt_by_mob()
}

struct SquidRandomMovementGoal {
    data: Arc<SquidData>,
}

impl Goal for SquidRandomMovementGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let mut rng = mob.get_random();
        if mob
            .get_mob_entity()
            .living_entity
            .no_action_time
            .load(Relaxed)
            > MAX_NO_ACTION_TIME
        {
            self.data.movement_vector.store(Vector3::default());
        } else if rng.random_range(0..self.get_tick_count(RANDOM_MOVEMENT_INTERVAL)) == 0
            || !entity.is_in_water()
            || self.data.movement_vector.load().length_squared() <= MOVEMENT_VECTOR_THRESHOLD
        {
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            self.data.movement_vector.store(Vector3::new(
                f64::from(cos(angle) * MOVEMENT_SPEED),
                f64::from(VERTICAL_MOVEMENT_OFFSET + rng.random::<f32>() * MOVEMENT_SPEED),
                f64::from(sin(angle) * MOVEMENT_SPEED),
            ));
        }
    }
}

struct SquidFleeGoal {
    data: Arc<SquidData>,
}

impl Goal for SquidFleeGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_entity().is_in_water()
            && last_hurt_by_mob(mob).is_some_and(|attacker| {
                (mob.get_entity().pos.load() - attacker.get_entity().pos.load()).length_squared()
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
        let mut flee_to = entity.pos.load() - attacker.get_entity().pos.load();
        let pos = (entity.pos.load() + flee_to).to_block_pos();
        let world = entity.world.load();
        let block = world.get_block(&pos);
        if world.get_fluid(&pos).has_tag(&tag::Fluid::MINECRAFT_WATER) || block.is_air() {
            let length = flee_to.length();
            if length > 0.0 {
                // Vanilla discards the normalized vector before scaling the original.
                let mut avoid_speed = SQUID_FLEE_SPEED;
                if length > SQUID_FLEE_MIN_DISTANCE {
                    avoid_speed -= (length - SQUID_FLEE_MIN_DISTANCE) / SQUID_FLEE_MIN_DISTANCE;
                }
                if avoid_speed > 0.0 {
                    flee_to = flee_to * avoid_speed;
                }
            }
            if block.is_air() {
                flee_to.y = 0.0;
            }
            self.data
                .movement_vector
                .store(flee_to / FLEE_VECTOR_DIVISOR);
        }
    }
}

#[macro_export]
macro_rules! squid_mob_hooks {
    () => {
        fn get_mob_entity(&self) -> &$crate::entity::mob::MobEntity { &self.mob_entity }
        fn as_ageable(&self) -> Option<&dyn $crate::entity::ageable::AgeableMob> { Some(self) }
        fn as_custom_sound(&self) -> Option<&dyn $crate::entity::custom_sound::CustomSound> { Some(self) }
        fn get_ambient_sound(&self) -> Option<pumpkin_data::sound::Sound> { Some($crate::entity::passive::squid::Squid::ambient_sound(self)) }
        fn get_ambient_sound_interval(&self) -> i32 { $crate::entity::passive::water_animal::AMBIENT_SOUND_INTERVAL }
        fn base_tick(&self) { $crate::entity::passive::squid::Squid::squid_base_tick(self, self); }
        fn travel(&self, caller: &dyn $crate::entity::EntityBase) -> bool {
            let entity = &self.mob_entity.living_entity.entity;
            entity.move_entity(caller, entity.velocity.load());
            true
        }
        fn mob_is_pushed_by_fluids(&self) -> bool { false }
        fn get_mob_gravity(&self) -> f64 { $crate::entity::passive::squid::DEFAULT_GRAVITY }
        fn check_spawn_obstruction(&self, world: &$crate::world::World) -> bool {
            $crate::entity::passive::ageable_water_creature::AgeableWaterCreature::water_creature_spawn_obstruction(self, world)
        }
        fn get_base_experience_reward(&self) -> u32 {
            $crate::entity::passive::ageable_water_creature::AgeableWaterCreature::water_creature_experience_reward(self)
        }
    };
}

macro_rules! impl_squid {
    ($squid:ty, ambient: $ambient:expr, squirt: $squirt:expr, ink: $ink:expr) => {
        impl AgeableWaterCreature for $squid {}
        impl AgeableMob for $squid {
            fn get_ageable_data(&self) -> &AgeableData {
                &self.ageable_data
            }
            fn age_boundary_reached(&self) {
                self.update_squid_dimensions();
            }
        }
        impl Squid for $squid {
            fn squid_data(&self) -> &SquidData {
                &self.squid_data
            }
            fn ambient_sound(&self) -> Sound {
                $ambient
            }
            fn squirt_sound(&self) -> Sound {
                $squirt
            }
            fn ink_particle(&self) -> Particle {
                $ink
            }
        }
        impl crate::entity::custom_sound::CustomSound for $squid {
            fn death_sound(&self) -> Option<Sound> {
                self.get_entity().entity_type.death_sound
            }
            fn hurt_sound(&self) -> Option<Sound> {
                self.get_entity().entity_type.hurt_sound
            }
            fn sound_category(&self) -> SoundCategory {
                SoundCategory::Neutral
            }
            fn sound_volume(&self) -> f32 {
                $crate::entity::passive::squid::SOUND_VOLUME
            }
        }
    };
}
pub(super) use impl_squid;

impl Mob for SquidEntity {
    crate::squid_mob_hooks!();
    fn mob_ai_step(&self) {
        self.squid_ai_step();
    }
    fn mob_init_data_tracker(&self) {
        self.init_squid_data();
    }
    fn on_damage_with_context(
        &self,
        _damage_type: pumpkin_data::damage::DamageType,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.squid_hurt()
    }
}

impl_squid!(SquidEntity, ambient: Sound::EntitySquidAmbient, squirt: Sound::EntitySquidSquirt, ink: Particle::SquidInk);
