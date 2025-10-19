use super::{Mob, MobEntity};
use crate::entity::ai::goal::look_around_goal::LookAroundGoal;
use crate::entity::ai::goal::move_to_target_pos_goal::MoveToTargetPos;
use crate::entity::ai::goal::step_and_destroy_block_goal::{StepAndDestroyBlockGoal, Stepping};
use crate::entity::ai::goal::zombie_attack_goal::ZombieAttackGoal;
use crate::entity::ai::goal::{Controls, Goal, ParentHandle};
use crate::entity::{
    Entity, NBTStorage,
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

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
            goal_selector.add_goal(2, ZombieAttackGoal::new(0.1, false));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for Zombie {}

impl Mob for Zombie {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

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

        this.step_and_destroy_block_goal.stepping = unsafe { ParentHandle::new(&this) };
        this.step_and_destroy_block_goal
            .move_to_target_pos_goal
            .move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }
}

#[async_trait]
impl Goal for DestroyEggGoal {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.can_start(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.step_and_destroy_block_goal.should_continue(mob).await
    }

    async fn start(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.start(mob).await;
    }

    async fn stop(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.stop(mob).await;
    }

    async fn tick(&mut self, mob: &dyn Mob) {
        self.step_and_destroy_block_goal.tick(mob).await;
    }

    fn should_run_every_tick(&self) -> bool {
        self.step_and_destroy_block_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.step_and_destroy_block_goal.controls()
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
