use super::{Goal, to_goal_ticks};
use crate::entity::ai::goal::move_to_target_pos_goal::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::mob::Mob;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};

const MAX_COOLDOWN: i32 = 20;

#[allow(dead_code)]
pub struct StepAndDestroyBlockGoal {
    stepping: Weak<dyn Stepping>,
    move_to_target_pos_goal: MoveToTargetPosGoal,
    target_block: &'static Block,
    counter: AtomicI32,
}

impl StepAndDestroyBlockGoal {
    // if stepping is set to None then will use itself for stepping
    #[must_use]
    pub fn new(
        stepping: Option<Weak<dyn Stepping>>,
        move_to_target_pos: Option<Weak<dyn MoveToTargetPos>>,
        target_block: &'static Block,
        speed: f64,
        max_y_difference: i32,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak_self: &Weak<Self>| {
            let weak_mtp: Weak<dyn MoveToTargetPos> =
                if let Some(move_to_target_pos) = move_to_target_pos {
                    move_to_target_pos
                } else {
                    weak_self.clone()
                };

            let move_to_target_pos_goal =
                MoveToTargetPosGoal::new(weak_mtp, speed, 24, max_y_difference);

            Self {
                stepping: stepping.unwrap_or_else(|| weak_self.clone()),
                move_to_target_pos_goal,
                target_block,
                counter: AtomicI32::new(0),
            }
        })
    }

    #[must_use]
    pub fn with_default(
        target_block: &'static Block,
        speed: f64,
        max_y_difference: i32,
    ) -> Arc<Self> {
        Self::new(None, None, target_block, speed, max_y_difference)
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
impl Goal for StepAndDestroyBlockGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.read().await;
        let level_info = world.level_info.read().await;
        if !level_info.game_rules.mob_griefing {
            false
        } else if self.move_to_target_pos_goal.cooldown.load(Relaxed) > 0 {
            self.move_to_target_pos_goal.cooldown.fetch_sub(1, Relaxed);
            false
        } else if self.move_to_target_pos_goal.find_target_pos(mob).await {
            self.move_to_target_pos_goal
                .cooldown
                .store(to_goal_ticks(MAX_COOLDOWN), Relaxed);
            true
        } else {
            self.move_to_target_pos_goal
                .cooldown
                .store(MoveToTargetPosGoal::get_interval(mob), Relaxed);
            false
        }
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob).await
    }

    async fn start(&self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob).await;
        self.counter.store(0, Relaxed);
    }

    async fn stop(&self, mob: &dyn Mob) {
        mob.get_mob_entity().living_entity.fall_distance.store(1.0);
    }

    async fn tick(&self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob).await;
        let mob_entity = mob.get_mob_entity();
        let world = mob.get_entity().world.read().await;
        let block_pos = mob.get_entity().block_pos.load();
        let tweak_pos = self.tweak_to_proper_pos(block_pos, world.clone()).await;
        // TODO: drop world?
        if !self.move_to_target_pos_goal.reached.load(Relaxed) || tweak_pos.is_none() {
            return;
        }
        let tweak_pos = tweak_pos.unwrap();
        let counter = self.counter.load(Relaxed);

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
                if let Some(stepping) = self.stepping.upgrade() {
                    stepping
                        .tick_stepping(
                            world.clone(),
                            self.move_to_target_pos_goal.target_pos.load(),
                        )
                        .await;
                } else {
                    self.tick_stepping(
                        world.clone(),
                        self.move_to_target_pos_goal.target_pos.load(),
                    )
                    .await;
                }
            }
        }

        if counter > 60 {
            // TODO: world.removeBlock HOW?
            // TODO: spawn particles
            self.on_destroy_block(world.clone(), tweak_pos).await;
        }

        self.counter.fetch_add(1, Relaxed);
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }
}

#[async_trait]
impl MoveToTargetPos for StepAndDestroyBlockGoal {
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
impl Stepping for StepAndDestroyBlockGoal {}
