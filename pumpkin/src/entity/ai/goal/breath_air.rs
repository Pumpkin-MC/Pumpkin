use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, passive::dolphin::DolphinEntity};
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

/// `BreathAirGoal.java`: below this air supply, actively swim toward the nearest air pocket
/// instead of relying on passive drowning damage.
const LOW_AIR_THRESHOLD: i32 = 140;
/// `BreathAirGoal.findAirPosition`: scans the column above the mob up to 8 blocks up.
const SEARCH_HEIGHT: i32 = 8;

/// Dolphin-specific port of `BreathAirGoal` (`Dolphin.java:158`).
///
/// This codebase has no per-mob air-supply tracking outside of `DolphinEntity` (see
/// `dolphin.rs`), so this downcasts rather than taking a generic `Mob`, matching
/// `DolphinSwimToTreasureGoal`'s existing pattern.
pub struct BreathAirGoal;

impl BreathAirGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }

    fn find_air_position(mob: &dyn Mob) -> BlockPos {
        let entity = mob.get_entity();
        let origin = entity.block_pos.load();
        let world = entity.world.load();

        // `BreathAirGoal.givesAir`: an empty fluid state (or a bubble column) with a non-solid
        // block, i.e. an air pocket the mob could occupy. Approximated without full
        // `PathComputationType.LAND` pathfindability, same simplification as
        // `TurtleGoToWaterGoal` uses for its water search.
        for dy in 0..=SEARCH_HEIGHT {
            let pos = BlockPos::new(origin.0.x, origin.0.y + dy, origin.0.z);
            let gives_air = if world.get_block(&pos) == &Block::BUBBLE_COLUMN {
                true
            } else {
                let (fluid, _) = world.get_fluid_and_fluid_state(&pos);
                fluid == &pumpkin_data::fluid::Fluid::EMPTY
                    && !world.get_block_state(&pos).is_solid()
            };
            if gives_air {
                return pos;
            }
        }

        BlockPos::new(origin.0.x, origin.0.y + SEARCH_HEIGHT, origin.0.z)
    }
}

impl Goal for BreathAirGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(dolphin) = mob.cast_any().downcast_ref::<DolphinEntity>() else {
                return false;
            };
            dolphin.get_air_supply() < LOW_AIR_THRESHOLD
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.can_start(mob)
    }

    /// Vanilla `BreathAirGoal.isInterruptable` returns `false`.
    fn can_stop(&self) -> bool {
        false
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            let target = Self::find_air_position(mob);
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(
                mob_pos,
                pumpkin_util::math::vector3::Vector3::new(
                    f64::from(target.0.x) + 0.5,
                    f64::from(target.0.y) + 1.0,
                    f64::from(target.0.z) + 0.5,
                ),
                1.0,
            ));
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = Self::find_air_position(mob);
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(
                mob_pos,
                pumpkin_util::math::vector3::Vector3::new(
                    f64::from(target.0.x) + 0.5,
                    f64::from(target.0.y) + 1.0,
                    f64::from(target.0.z) + 0.5,
                ),
                1.0,
            ));
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
