use super::{Mob, MobEntity};
use crate::entity::ai::goal::look_around::LookAroundGoal;
use crate::entity::ai::goal::move_to_target_pos::MoveToTargetPos;
use crate::entity::ai::goal::step_and_destroy_block::{
    StepAndDestroyBlockGoal, Stepping, SteppingFuture,
};
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture, ParentHandle};
use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use rand::{RngExt, rng};
use std::pin::Pin;
use std::sync::{Arc, Weak};

pub mod drowned;
pub mod husk;

pub struct ZombieEntity {
    pub mob_entity: MobEntity,
}

impl ZombieEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
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
            goal_selector.add_goal(3, ZombieAttackGoal::new(1.0, false));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for ZombieEntity {}

impl Mob for ZombieEntity {
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

impl Goal for DestroyEggGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.step_and_destroy_block_goal.can_start(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.step_and_destroy_block_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.step_and_destroy_block_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.step_and_destroy_block_goal.stop(mob).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.step_and_destroy_block_goal.tick(mob).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        self.step_and_destroy_block_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.step_and_destroy_block_goal.controls()
    }
}

impl Stepping for DestroyEggGoal {
    fn tick_stepping(&self, world: Arc<World>, block_pos: BlockPos) -> SteppingFuture<'_> {
        Box::pin(async move {
            let random = rng().random::<f32>();

            // NOTE: block_pos.0.to_f64() is assumed to be the correct way to get Vector3<f64>
            let pos_f64 = (block_pos.0).to_f64();

            world
                .play_sound_raw(
                    Sound::EntityZombieDestroyEgg as u16,
                    SoundCategory::Hostile,
                    &pos_f64,
                    0.7,
                    random.mul_add(0.2, 0.9),
                )
                .await;
        })
    }

    fn on_destroy_block(&self, world: Arc<World>, block_pos: BlockPos) -> SteppingFuture<'_> {
        Box::pin(async move {
            let random = rng().random::<f32>();

            // NOTE: block_pos.0.to_f64() is assumed to be the correct way to get Vector3<f64>
            let pos_f64 = (block_pos.0).to_f64();

            world
                .play_sound_raw(
                    Sound::EntityTurtleEggBreak as u16,
                    SoundCategory::Blocks,
                    &pos_f64,
                    0.7,
                    random.mul_add(0.2, 0.9),
                )
                .await;
        })
    }
}

impl MoveToTargetPos for DestroyEggGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            self.step_and_destroy_block_goal
                .is_target_pos(world, block_pos)
                .await
        })
    }

    fn get_desired_distance_to_target(&self) -> f64 {
        1.14
    }
}
