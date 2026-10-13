//! What vanilla's `AbstractFish` and `WaterAnimal` give cod, salmon, pufferfish and tropical fish.

use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::ai::control::fish_move_control::FishMoveControl;
use crate::entity::ai::goal::avoid_entity::AvoidEntityGoal;
use crate::entity::ai::goal::escape_danger::EscapeDangerGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::pathfinder::Navigator;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::mob::{Mob, MobEntity};

const MAX_AIR: i32 = 300;

/// Air a fish has left out of water.
pub struct FishAir(AtomicI32);

impl Default for FishAir {
    fn default() -> Self {
        Self(AtomicI32::new(MAX_AIR))
    }
}

/// Swims in water, panics when hurt and keeps clear of players.
pub fn init(mob: &MobEntity) {
    let mut navigator = Navigator::water_bound(false);
    navigator.set_pathfinding_malus(PathType::Water, 0.0);
    *mob.navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = navigator;
    *mob.move_control
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Box::new(FishMoveControl::default());

    let mut goals = mob
        .goals_selector
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    goals.add_goal(0, EscapeDangerGoal::new(1.25));
    goals.add_goal(
        2,
        Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
    );
    goals.add_goal(4, Box::new(WanderAroundGoal::swimming(1.0, 40)));
}

/// Out of water a fish runs out of air and flops about; vanilla `WaterAnimal.handleAirSupply` and
/// `AbstractFish.aiStep`.
pub fn tick(mob: &dyn Mob, caller: &dyn EntityBase, air: &FishAir, flop: Sound) {
    let living = &mob.get_mob_entity().living_entity;
    let entity = &living.entity;
    if entity.touching_water.load(Ordering::Relaxed) || living.health.load() <= 0.0 {
        air.0.store(MAX_AIR, Ordering::Relaxed);
        return;
    }

    if air.0.fetch_sub(1, Ordering::Relaxed) - 1 == -20 {
        air.0.store(0, Ordering::Relaxed);
        caller.damage(caller, 2.0, DamageType::DROWN);
    }

    if entity.on_ground.load(Ordering::Relaxed) {
        let mut rng = mob.get_random();
        let mut nudge = || f64::from((rng.random::<f32>() * 2.0 - 1.0) * 0.05);
        let hop = Vector3::new(nudge(), 0.4, nudge());
        let pitch = (rng.random::<f32>() - rng.random::<f32>()) * 0.2 + 1.0;
        entity.on_ground.store(false, Ordering::Relaxed);
        entity.set_velocity(entity.velocity.load() + hop);
        entity.world.load().play_sound_fine(
            flop,
            SoundCategory::Neutral,
            &entity.pos.load(),
            1.0,
            pitch,
        );
    }
}

/// Vanilla `AbstractFish.travel` in water: a slow sink instead of gravity, and no climbing out at
/// the shore the way walking mobs do.
pub fn travel_in_water(mob: &dyn Mob, caller: &dyn EntityBase) -> bool {
    let living = &mob.get_mob_entity().living_entity;
    let entity = &living.entity;
    let input = living.movement_input.load();
    entity.update_velocity_from_input(Vector3::new(input.x, 0.0, input.z), 0.01);
    entity.move_entity(caller, entity.velocity.load());

    // A fish never has a target, so it always takes vanilla's targetless sink.
    let velocity = entity.velocity.load() * 0.9;
    entity
        .velocity
        .store(Vector3::new(velocity.x, velocity.y - 0.005, velocity.z));
    true
}
