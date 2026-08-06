use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, GoalFuture, ParentHandle};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

/// Vanilla: `Strider.StriderGoToLavaGoal` (`Strider.java:484-516`), a `MoveToBlockGoal`
/// subclass (`super(strider, speedModifier, 8, 2)`) that paths a stranded strider back onto
/// lava.
pub struct StriderGoToLavaGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
}

impl StriderGoToLavaGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        let mut this = Box::new(Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 8, 2),
        });

        // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }

    fn in_lava(mob: &dyn Mob) -> bool {
        mob.get_entity().touching_lava.load(Ordering::SeqCst)
    }

    fn is_target_valid(world: &Arc<World>, block_pos: BlockPos) -> bool {
        let block = world.get_block(&block_pos);
        if block != &Block::LAVA {
            return false;
        }
        // Approximates vanilla's `isPathfindable(PathComputationType.LAND)`: the block a
        // strider would stand on above the lava must not be a solid obstruction.
        !world.get_block_state(&block_pos.up()).is_solid()
    }
}

impl Goal for StriderGoToLavaGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if Self::in_lava(mob) {
                return false;
            }
            self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if Self::in_lava(mob) {
                return false;
            }
            let world = mob.get_entity().world.load_full();
            let target = self.move_to_target_pos_goal.target_pos;
            Self::is_target_valid(&world, target)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.move_to_target_pos_goal.start(mob).await })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.move_to_target_pos_goal.stop(mob).await })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.move_to_target_pos_goal.tick(mob).await })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

impl MoveToTargetPos for StriderGoToLavaGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move { Self::is_target_valid(&world, block_pos) })
    }
}
