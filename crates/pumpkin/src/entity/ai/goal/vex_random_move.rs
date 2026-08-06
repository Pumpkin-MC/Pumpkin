use std::sync::Weak;

use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::vex::VexEntity;

/// Vanilla: `Vex.VexRandomMoveGoal`.
///
/// Rolls a random destination around `bound_origin` (falling back to the vex's current position)
/// up to 3 times per activation, taking the first passable one it finds. This goal never
/// "continues" -- `canContinueToUse` is always `false` -- so it re-rolls independently every
/// eligible tick rather than persisting across ticks.
pub struct VexRandomMoveGoal {
    vex: Weak<VexEntity>,
}

impl VexRandomMoveGoal {
    #[must_use]
    pub const fn new(vex: Weak<VexEntity>) -> Self {
        Self { vex }
    }
}

impl Goal for VexRandomMoveGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return false;
            };
            !vex.mob_entity.move_control.lock().unwrap().has_wanted()
                && rand::rng().random_range(0..to_goal_ticks(7)) == 0
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return;
            };
            let origin = vex
                .bound_origin()
                .unwrap_or_else(|| vex.mob_entity.living_entity.entity.block_pos.load());
            let world = vex.mob_entity.living_entity.entity.world.load();
            let has_target = vex.mob_entity.target.lock().await.is_some();

            let mut rng = rand::rng();
            for _ in 0..3 {
                let test_pos = BlockPos::new(
                    origin.0.x + rng.random_range(0..15) - 7,
                    origin.0.y + rng.random_range(0..11) - 5,
                    origin.0.z + rng.random_range(0..15) - 7,
                );
                if world.get_block_state(&test_pos).is_air() {
                    vex.mob_entity
                        .move_control
                        .lock()
                        .unwrap()
                        .set_wanted_position(
                            f64::from(test_pos.0.x) + 0.5,
                            f64::from(test_pos.0.y) + 0.5,
                            f64::from(test_pos.0.z) + 0.5,
                            0.25,
                        );
                    if !has_target {
                        vex.mob_entity
                            .look_control
                            .lock()
                            .unwrap()
                            .look_at_with_range(
                                f64::from(test_pos.0.x) + 0.5,
                                f64::from(test_pos.0.y) + 0.5,
                                f64::from(test_pos.0.z) + 0.5,
                                180.0,
                                20.0,
                            );
                    }
                    break;
                }
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
