use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use crate::entity::{
    EntityBase,
    ai::{
        control::fish_move_control::FishMoveControl,
        goal::{Controls, Goal, random_swimming::RandomSwimmingGoal},
        pathfinder::{Navigator, node::PathType},
    },
    mob::{Mob, MobEntity},
};
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::water_animal::{AMBIENT_SOUND_INTERVAL, WaterAnimal};

#[derive(Default)]
pub struct FishData {
    pub from_bucket: AtomicBool,
    ambient_sound_time: AtomicI32,
}

const WATER_ACCELERATION: f32 = 0.01;
const WATER_FRICTION: f64 = 0.9;
const SINK_SPEED: f64 = 0.005;
const FLOP_HORIZONTAL: f32 = 0.05;
const FLOP_VERTICAL: f32 = 0.4;

pub fn initialize_fish_movement(mob: &mut MobEntity) {
    let navigator = mob
        .navigator
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *navigator = Navigator::water_bound(false);
    navigator.set_pathfinding_malus(PathType::Water, 0.0);
    *mob.move_control
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Box::new(FishMoveControl::default());
}

const PANIC_SPEED: f64 = 1.25;
const AVOID_PLAYER_RANGE: f64 = 8.0;
const AVOID_PLAYER_WALK_SPEED: f64 = 1.6;
const AVOID_PLAYER_SPRINT_SPEED: f64 = 1.4;
const RANDOM_SWIM_SPEED: f64 = 1.0;
const RANDOM_SWIM_INTERVAL: i32 = 40;

pub fn register_fish_goals(mob: &MobEntity) {
    use crate::entity::ai::goal::{avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal};
    let goals = &mob.goals_selector;
    goals.add_goal(0, EscapeDangerGoal::new(PANIC_SPEED));
    goals.add_goal(
        2,
        Box::new(AvoidEntityGoal::new(
            &pumpkin_data::entity::EntityType::PLAYER,
            AVOID_PLAYER_RANGE,
            AVOID_PLAYER_WALK_SPEED,
            AVOID_PLAYER_SPRINT_SPEED,
        )),
    );
    goals.add_goal(4, Box::new(FishSwimGoal::default()));
}

struct FishSwimGoal {
    inner: RandomSwimmingGoal,
}

impl Default for FishSwimGoal {
    fn default() -> Self {
        Self {
            inner: RandomSwimmingGoal::new(RANDOM_SWIM_SPEED, RANDOM_SWIM_INTERVAL),
        }
    }
}

impl Goal for FishSwimGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.as_fish().is_some_and(AbstractFish::can_random_swim) && self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub trait AbstractFish: WaterAnimal + crate::entity::bucketable::Bucketable {
    fn flop_sound(&self) -> Sound;

    fn can_random_swim(&self) -> bool {
        self.as_schooling_fish()
            .is_none_or(|fish| !fish.is_follower())
    }

    fn fish_travel_in_water(&self, caller: &dyn EntityBase) {
        let mob = self.get_mob_entity();
        let living = &mob.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(WATER_ACCELERATION),
        );
        living.make_move(caller);
        let mut velocity = entity.velocity.load() * WATER_FRICTION;
        if mob
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            velocity.y -= SINK_SPEED;
        }
        entity.velocity.store(velocity);
    }

    fn fish_ai_step(&self) {
        let entity = self.get_entity();
        if !entity.is_in_water()
            && entity.on_ground.load(Relaxed)
            && entity.vertical_collision.load(Relaxed)
        {
            let mut rng = self.get_random();
            entity.velocity.store(
                entity.velocity.load()
                    + Vector3::new(
                        f64::from((rng.random::<f32>() * 2.0 - 1.0) * FLOP_HORIZONTAL),
                        f64::from(FLOP_VERTICAL),
                        f64::from((rng.random::<f32>() * 2.0 - 1.0) * FLOP_HORIZONTAL),
                    ),
            );
            entity.on_ground.store(false, Relaxed);
            entity.velocity_dirty.store(true, Relaxed);
            if !entity.is_silent() {
                entity.world.load().play_sound_raw(
                    self.flop_sound() as u16,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    self.get_mob_entity().living_entity.get_pitch(),
                );
            }
        }
    }

    fn fish_data(&self) -> &FishData;

    fn ambient_sound(&self) -> Sound;

    fn fish_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("FromBucket", self.is_from_bucket());
    }

    fn fish_read_nbt(&self, nbt: &NbtCompound) {
        self.set_from_bucket(nbt.get_bool("FromBucket").unwrap_or(false));
    }

    fn play_ambient_sound(&self) {
        let entity = self.get_entity();
        if !entity.is_silent() {
            entity.world.load().play_sound_raw(
                self.ambient_sound() as u16,
                pumpkin_data::sound::SoundCategory::Neutral,
                &entity.pos.load(),
                1.0,
                self.get_mob_entity().living_entity.get_pitch(),
            );
        }
    }

    fn fish_base_tick(&self, caller: &dyn crate::entity::EntityBase) {
        if !self.get_mob_entity().living_entity.is_dead_or_dying()
            && self.get_random().random_range(0..1000)
                < self.fish_data().ambient_sound_time.fetch_add(1, Relaxed)
        {
            self.reset_ambient_sound_time();
            self.play_ambient_sound();
        }
        self.handle_air_supply(caller);
    }

    fn reset_ambient_sound_time(&self) {
        self.fish_data()
            .ambient_sound_time
            .store(-AMBIENT_SOUND_INTERVAL, Relaxed);
    }
}
