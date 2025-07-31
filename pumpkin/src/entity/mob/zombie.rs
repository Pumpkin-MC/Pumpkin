use super::{Mob, MobEntity};
use crate::entity::ai::goal::look_around_goal::LookAroundGoal;
use crate::entity::ai::goal::move_to_target_pos_goal::MoveToTargetPos;
use crate::entity::ai::goal::step_and_destroy_block_goal::{StepAndDestroyBlockGoal, Stepping};
use crate::entity::ai::goal::zombie_attack_goal::ZombieAttackGoal;
use crate::entity::ai::goal::{Goal, GoalControl};
use crate::entity::{
    Entity,
    ai::goal::{active_target_goal::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use rand::{Rng, rng};
use std::sync::{Arc, Weak};

pub struct Zombie {
    mob_entity: MobEntity,
}

impl Zombie {
    pub async fn make(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zombie = Self { mob_entity };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // This is needed for goals because some of them needs the MobEntity fully initialized in the constructor
        // The Weak is stored to avoid memory leak and can be used if and where necessary
        let goal_selector = &mob_arc.mob_entity.goals_selector;
        let target_selector = &mob_arc.mob_entity.target_selector;

        goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3)).await;
        goal_selector
            .add_goal(
                8,
                Arc::new(LookAtEntityGoal::with_default(
                    mob_weak,
                    &EntityType::PLAYER,
                    8.0,
                )),
            )
            .await;
        goal_selector
            .add_goal(8, Arc::new(LookAroundGoal::default()))
            .await;
        goal_selector
            .add_goal(2, Arc::new(ZombieAttackGoal::new(0.1, false)))
            .await;

        target_selector
            .add_goal(
                2,
                Arc::new(
                    ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true)
                        .await,
                ),
            )
            .await;

        mob_arc
    }
}

impl Mob for Zombie {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

pub struct DestroyEggGoal {
    step_and_destroy_block_goal: Arc<StepAndDestroyBlockGoal>,
}

impl DestroyEggGoal {
    #[must_use]
    pub fn new(speed: f64, max_y_difference: i32) -> Arc<Self> {
        Arc::new_cyclic(|weak: &Weak<Self>| {
            let step_and_destroy_block_goal = StepAndDestroyBlockGoal::new(
                Some(weak.clone()),
                Some(weak.clone()),
                &Block::TURTLE_EGG,
                speed,
                max_y_difference,
            );
            Self {
                step_and_destroy_block_goal,
            }
        })
    }
}

#[async_trait]
impl Goal for DestroyEggGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.can_start(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.should_continue(mob).await
    }

    async fn start(&self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.start(mob).await;
    }

    async fn stop(&self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.stop(mob).await;
    }

    async fn tick(&self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.tick(mob).await;
    }

    fn should_run_every_tick(&self) -> bool {
        self.step_and_destroy_block_goal.should_run_every_tick()
    }

    fn get_goal_control(&self) -> &GoalControl {
        self.step_and_destroy_block_goal.get_goal_control()
    }
}

#[async_trait]
impl Stepping for DestroyEggGoal {
    async fn tick_stepping(&self, world: Arc<World>, block_pos: BlockPos) {
        let random = rng().random::<f32>();
        world
            .play_sound_raw(
                Sound::EntityZombieDestroyEgg as u16,
                SoundCategory::Hostile,
                &block_pos.0.to_f64(),
                0.7,
                0.9 + random * 0.2,
            )
            .await;
    }

    async fn on_destroy_block(&self, world: Arc<World>, block_pos: BlockPos) {
        let random = rng().random::<f32>();
        world
            .play_sound_raw(
                Sound::EntityTurtleEggBreak as u16,
                SoundCategory::Blocks,
                &block_pos.0.to_f64(),
                0.7,
                0.9 + random * 0.2,
            )
            .await;
    }
}

#[async_trait]
impl MoveToTargetPos for DestroyEggGoal {
    async fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool {
        self.step_and_destroy_block_goal
            .is_target_pos(world, block_pos)
            .await
    }

    fn get_desired_distance_to_target(&self) -> f64 {
        1.14
    }
}
