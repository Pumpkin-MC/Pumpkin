//! Port of `behavior/AnimalPanic.java`.
//!
//! This is the behavior that proves the split-lock design: its `HURT_BY` gate is populated from
//! `LivingEntity::damage_with_context`, i.e. from projectile/block/fluid call sites that run
//! outside the mob's own AI tick. If the memory store were taken out of its mutex for the tick
//! the way `GoalSelector` is, those writes would land on a throwaway `Default` and be lost.
//!
//! DEVIATION: `LandRandomPos.getPos(mob, 5, 4)` is still represented by a uniform random offset
//! in the same +/-5 horizontal, +/-4 vertical box and lets the navigator reject unreachable
//! destinations. The source's on-fire water search is ported below.

use rand::RngExt;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::ai::brain::memory::{
    HurtByMemory, IsPanickingMemory, MemoryKeyId, MemoryStatus, PositionTracker, WalkTarget,
    WalkTargetMemory,
};
use crate::entity::mob::Mob;
use crate::world::World;

/// `AnimalPanic.PANIC_DISTANCE_HORIZONTAL` / `PANIC_DISTANCE_VERTICAL` (`:29-30`).
const PANIC_DISTANCE_HORIZONTAL: f64 = 5.0;
const PANIC_DISTANCE_VERTICAL: f64 = 4.0;

/// Vanilla `BlockPos.findClosestMatch(center, 5, 1, water)` using the same Manhattan traversal
/// order as `BlockPos.withinManhattan`. The search is only used when the mob is on fire and its
/// current block has no collision shape (`AnimalPanic.java:89-98`).
fn nearest_water(world: &World, origin: BlockPos) -> Option<BlockPos> {
    if !world.get_block_state(&origin).collision_shapes.is_empty() {
        return None;
    }

    for depth in 0i32..=11 {
        let max_x: i32 = 5.min(depth);
        for x in -max_x..=max_x {
            let max_y: i32 = 1.min(depth - x.abs());
            for y in -max_y..=max_y {
                let z = depth - x.abs() - y.abs();
                if z > 5 {
                    continue;
                }

                let candidate = BlockPos::new(origin.0.x + x, origin.0.y + y, origin.0.z + z);
                let (fluid, _) = world.get_fluid_and_fluid_state(&candidate);
                if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                    return Some(candidate);
                }
                if z != 0 {
                    let candidate = BlockPos::new(origin.0.x + x, origin.0.y + y, origin.0.z - z);
                    let (fluid, _) = world.get_fluid_and_fluid_state(&candidate);
                    if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
}

pub struct AnimalPanic {
    speed_multiplier: f32,
}

impl AnimalPanic {
    /// `new AnimalPanic(speedMultiplier)` (`AnimalPanic.java:35-37`), whose entry condition is
    /// `IS_PANICKING` REGISTERED + `HURT_BY` REGISTERED and whose duration is 100..120 (`:54`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(speed_multiplier: f32) -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::with_duration(
            Self { speed_multiplier },
            vec![
                (MemoryKeyId::IsPanicking, MemoryStatus::Registered),
                (MemoryKeyId::HurtBy, MemoryStatus::Registered),
            ],
            100,
            120,
        ))
    }
}

impl TimedBehavior for AnimalPanic {
    fn debug_name(&self) -> &'static str {
        "AnimalPanic"
    }

    /// `checkExtraStartConditions` (`AnimalPanic.java:60-63`): the recorded damage type is in
    /// `DamageTypeTags.PANIC_CAUSES`, or the mob is already flagged as panicking.
    fn check_extra_start_conditions(&mut self, _mob: &dyn Mob, brain: &Brain) -> bool {
        let hurt_by_panics = brain.get::<HurtByMemory>().is_some_and(|damage_type| {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_PANIC_CAUSES)
        });
        hurt_by_panics || brain.has_value::<IsPanickingMemory>()
    }

    /// `canStillUse` (`AnimalPanic.java:65-67`) is unconditionally true; the 100..120 duration
    /// is what ends the panic.
    fn can_still_use(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) -> bool {
        true
    }

    /// `start` (`AnimalPanic.java:69-73`).
    fn start(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.set::<IsPanickingMemory>(true);
        brain.erase::<WalkTargetMemory>();
        mob.get_mob_entity().navigator.lock().unwrap().stop();
    }

    /// `stop` (`AnimalPanic.java:75-78`).
    fn stop(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.erase::<IsPanickingMemory>();
    }

    /// `tick` (`AnimalPanic.java:80-87`): only pick a new flee destination once the navigator
    /// has run out of path, so the mob does not re-roll every tick.
    fn tick(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        let navigation_done = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
        if !navigation_done {
            return;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let panic_to = if entity
            .has_visual_fire
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            nearest_water(&world, entity.block_pos.load()).map(|pos| pos.to_f64())
        } else {
            None
        }
        .unwrap_or_else(|| {
            let pos = entity.pos.load();
            let mut rng = mob.get_random();
            Vector3::new(
                pos.x + rng.random_range(-PANIC_DISTANCE_HORIZONTAL..=PANIC_DISTANCE_HORIZONTAL),
                pos.y + rng.random_range(-PANIC_DISTANCE_VERTICAL..=PANIC_DISTANCE_VERTICAL),
                pos.z + rng.random_range(-PANIC_DISTANCE_HORIZONTAL..=PANIC_DISTANCE_HORIZONTAL),
            )
        });

        brain.set::<WalkTargetMemory>(WalkTarget::new(
            PositionTracker::of_position(panic_to),
            self.speed_multiplier,
            0,
        ));
    }
}
