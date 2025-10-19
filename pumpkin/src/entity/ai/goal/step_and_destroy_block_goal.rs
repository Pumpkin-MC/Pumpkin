use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::ParentHandle;
use crate::entity::ai::goal::move_to_target_pos_goal::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::mob::Mob;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

const MAX_COOLDOWN: i32 = 20;

#[allow(dead_code)]
pub struct StepAndDestroyBlockGoal<S: Stepping, M: MoveToTargetPos> {
    pub stepping: ParentHandle<S>,
    pub move_to_target_pos_goal: MoveToTargetPosGoal<M>,
    target_block: &'static Block,
    counter: i32,
}

impl<S: Stepping, M: MoveToTargetPos> StepAndDestroyBlockGoal<S, M> {
    // if stepping is set to None then will use itself for stepping
    #[must_use]
    pub fn new(
        stepping: ParentHandle<S>,
        move_to_target_pos: ParentHandle<M>,
        target_block: &'static Block,
        speed: f64,
        max_y_difference: i32,
    ) -> Self {
        let move_to_target_pos_goal =
            MoveToTargetPosGoal::new(move_to_target_pos, speed, 24, max_y_difference);

        Self {
            stepping,
            move_to_target_pos_goal,
            target_block,
            counter: 0,
        }
    }

    #[must_use]
    pub fn with_default(target_block: &'static Block, speed: f64, max_y_difference: i32) -> Self {
        Self::new(
            ParentHandle::none(),
            ParentHandle::none(),
            target_block,
            speed,
            max_y_difference,
        )
    }
    async fn tweak_to_proper_pos(&self, pos: BlockPos, world: Arc<World>) -> Option<BlockPos> {
        if world.get_block(&pos).await.id == self.target_block.id {
            Some(pos)
        } else {
            let block_pos = [
                pos.down(),
                pos.west(),
                pos.east(),
                pos.north(),
                pos.south(),
                pos.down().down(),
            ];

            for pos in block_pos {
                if world.get_block(&pos).await.id == self.target_block.id {
                    return Some(pos);
                }
            }
            None
        }
    }
}

// Contains overridable functions
#[async_trait]
pub trait Stepping: Send + Sync {
    async fn tick_stepping(&self, _world: Arc<World>, _block_pos: BlockPos) {}

    async fn on_destroy_block(&self, _world: Arc<World>, _block_pos: BlockPos) {}
}

#[async_trait]
impl<S: Stepping, M: MoveToTargetPos> Goal for StepAndDestroyBlockGoal<S, M> {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let world = &mob.get_entity().world;
        let level_info = world.level_info.read().await;
        if !level_info.game_rules.mob_griefing {
            false
        } else if self.move_to_target_pos_goal.cooldown > 0 {
            self.move_to_target_pos_goal.cooldown -= 1;
            false
        } else if self.move_to_target_pos_goal.find_target_pos(mob).await {
            self.move_to_target_pos_goal.cooldown = to_goal_ticks(MAX_COOLDOWN);
            true
        } else {
            self.move_to_target_pos_goal.cooldown = MoveToTargetPosGoal::<M>::get_interval(mob);
            false
        }
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob).await
    }

    async fn start(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob).await;
        self.counter = 0;
    }

    async fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().living_entity.fall_distance.store(1.0);
    }

    async fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob).await;
        let mob_entity = mob.get_mob_entity();
        let world = &mob.get_entity().world;
        let block_pos = mob.get_entity().block_pos.load();
        let tweak_pos = self.tweak_to_proper_pos(block_pos, world.clone()).await;
        // TODO: drop world?
        if !self.move_to_target_pos_goal.reached || tweak_pos.is_none() {
            return;
        }
        let tweak_pos = tweak_pos.unwrap();
        let counter = self.counter;

        if counter > 0 {
            let velocity = mob_entity.living_entity.entity.velocity.load();
            mob_entity
                .living_entity
                .entity
                .set_velocity(Vector3::new(velocity.x, 0.3, velocity.z))
                .await;
            // TODO: spawn particles
        }

        if counter % 2 == 0 {
            let velocity = mob_entity.living_entity.entity.velocity.load();
            mob_entity
                .living_entity
                .entity
                .set_velocity(Vector3::new(velocity.x, -0.3, velocity.z))
                .await;
            if counter % 6 == 0 {
                if let Some(stepping) = self.stepping.get() {
                    stepping
                        .tick_stepping(world.clone(), self.move_to_target_pos_goal.target_pos)
                        .await;
                } else {
                    self.tick_stepping(world.clone(), self.move_to_target_pos_goal.target_pos)
                        .await;
                }
            }
        }

        if counter > 60 {
            // TODO: world.removeBlock HOW?
            // TODO: spawn particles
            self.on_destroy_block(world.clone(), tweak_pos).await;
        }

        self.counter += 1;
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

#[async_trait]
impl<S: Stepping, M: MoveToTargetPos> MoveToTargetPos for StepAndDestroyBlockGoal<S, M> {
    async fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool {
        world.get_block(&block_pos).await.id == self.target_block.id
            && world.get_block_state(&block_pos.up()).await.is_air()
            && world
                .get_block_state(&block_pos.up_height(2))
                .await
                .is_air()
    }
}

#[async_trait]
impl<S: Stepping, M: MoveToTargetPos> Stepping for StepAndDestroyBlockGoal<S, M> {}
