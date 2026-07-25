use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

use crate::entity::ai::goal::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::ai::goal::{Controls, Goal, GoalFuture, ParentHandle};
use crate::entity::mob::Mob;
use crate::world::World;

/// Moves a strider to a nearby lava source while it is out of lava.
pub struct GoToLavaGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
}

impl GoToLavaGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        let mut goal = Box::new(Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 8, 2),
        });

        goal.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&goal) };
        goal
    }

    fn is_in_lava(mob: &dyn Mob) -> bool {
        mob.get_entity()
            .touching_lava
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl MoveToTargetPos for GoToLavaGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            Block::from_state_id(world.get_block_state(&block_pos).id) == &Block::LAVA
                && !world.get_block_state(&block_pos.up()).is_solid()
        })
    }

    fn get_move_to_target(&self, block_pos: BlockPos) -> BlockPos {
        block_pos
    }

    fn should_recalculate_path(&self, trying_time: i32) -> bool {
        trying_time % 20 == 0
    }
}

impl Goal for GoToLavaGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            !Self::is_in_lava(mob) && self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            !Self::is_in_lava(mob) && self.move_to_target_pos_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.move_to_target_pos_goal.start(mob).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.move_to_target_pos_goal.tick(mob).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}
