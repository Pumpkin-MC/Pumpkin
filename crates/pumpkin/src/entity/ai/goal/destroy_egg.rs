use crate::entity::ai::goal::move_to_target_pos::MoveToTargetPos;
use crate::entity::ai::goal::step_and_destroy_block::{StepAndDestroyBlockGoal, Stepping};
use crate::entity::ai::goal::{Controls, Goal, ParentHandle};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use rand::{RngExt, rng};
use std::sync::Arc;

pub struct DestroyEggGoal {
    step_and_destroy_block_goal: StepAndDestroyBlockGoal<Self, Self>,
}

impl DestroyEggGoal {
    #[must_use]
    pub fn new(speed: f64, max_y_difference: i32) -> Box<Self> {
        let mut this = Box::new(Self {
            step_and_destroy_block_goal: StepAndDestroyBlockGoal::new(
                ParentHandle::none(),
                ParentHandle::none(),
                &Block::TURTLE_EGG,
                speed,
                max_y_difference,
            ),
        });

        // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
        this.step_and_destroy_block_goal.stepping = unsafe { ParentHandle::new(&this) };
        this.step_and_destroy_block_goal.move_to_target_pos_goal.move_to_target_pos =
            // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
            unsafe { ParentHandle::new(&this) };

        this
    }
}

impl Goal for DestroyEggGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.can_start(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.step_and_destroy_block_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.step_and_destroy_block_goal.controls()
    }
}

impl Stepping for DestroyEggGoal {
    fn tick_stepping(&self, world: Arc<World>, block_pos: BlockPos) {
        let random = rng().random::<f32>();
        let pos_f64 = (block_pos.0).to_f64();
        world.play_sound_raw(
            Sound::EntityZombieDestroyEgg as u16,
            SoundCategory::Hostile,
            &pos_f64,
            0.7,
            random.mul_add(0.2, 0.9),
        );
    }

    fn on_destroy_block(&self, world: Arc<World>, block_pos: BlockPos) {
        let random = rng().random::<f32>();
        let pos_f64 = (block_pos.0).to_f64();
        world.play_sound_raw(
            Sound::EntityTurtleEggBreak as u16,
            SoundCategory::Blocks,
            &pos_f64,
            0.7,
            random.mul_add(0.2, 0.9),
        );
    }
}

impl MoveToTargetPos for DestroyEggGoal {
    fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool {
        self.step_and_destroy_block_goal
            .is_target_pos(world, block_pos)
    }

    fn get_desired_distance_to_target(&self) -> f64 {
        1.14
    }
}
