use std::pin::Pin;
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, TurtleEggLikeProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, GoalFuture, ParentHandle, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::passive::turtle::TurtleEntity;
use crate::world::World;

/// `Turtle.layEggCounter` ticks the laying animation runs before the egg is actually placed.
const LAY_EGG_TICKS: i32 = 200;
/// `Turtle.TurtleLayEggGoal`: `MoveToBlockGoal(turtle, speedModifier, 16)`.
const SEARCH_RANGE: i32 = 16;
/// `Turtle.setInLoveTime`: cooldown applied after a successful lay so the turtle doesn't
/// immediately breed/lay again.
const POST_LAY_LOVE_COOLDOWN: i32 = 600;

/// Vanilla: `Turtle.TurtleLayEggGoal` (`Turtle.java:433-480`), a `MoveToBlockGoal` subclass
/// that walks a gravid turtle back to a sand block near its home and places a `TURTLE_EGG`.
pub struct TurtleLayEggGoal {
    turtle: Weak<TurtleEntity>,
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
    lay_egg_counter: i32,
}

impl TurtleLayEggGoal {
    #[must_use]
    pub fn new(turtle: Weak<TurtleEntity>, speed: f64) -> Box<Self> {
        let mut this = Box::new(Self {
            turtle,
            move_to_target_pos_goal: MoveToTargetPosGoal::with_default(
                ParentHandle::none(),
                speed,
                SEARCH_RANGE,
            ),
            lay_egg_counter: 0,
        });

        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }

    /// `Turtle.TurtleLayEggGoal.canUse`/`canContinueToUse`: only while carrying an egg and
    /// within 9 blocks of home.
    fn near_home_with_egg(&self, mob: &dyn Mob) -> bool {
        let Some(turtle) = self.turtle.upgrade() else {
            return false;
        };
        if !turtle.has_egg() {
            return false;
        }
        let Some(home) = turtle.get_home() else {
            return false;
        };
        let my_pos = mob.get_entity().pos.load();
        my_pos.squared_distance_to_vec(&home.to_centered_f64()) < 9.0 * 9.0
    }
}

impl Goal for TurtleLayEggGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !self.near_home_with_egg(mob) {
                return false;
            }
            self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.near_home_with_egg(mob) && self.move_to_target_pos_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.move_to_target_pos_goal.start(mob).await })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.move_to_target_pos_goal.stop(mob).await;
            if let Some(turtle) = self.turtle.upgrade() {
                turtle.set_laying_egg(false);
            }
            self.lay_egg_counter = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.move_to_target_pos_goal.tick(mob).await;

            let Some(turtle) = self.turtle.upgrade() else {
                return;
            };
            let entity = mob.get_entity();
            if entity
                .touching_water
                .load(std::sync::atomic::Ordering::SeqCst)
                || !self.move_to_target_pos_goal.reached
            {
                return;
            }

            if self.lay_egg_counter < 1 {
                turtle.set_laying_egg(true);
            } else if self.lay_egg_counter > to_goal_ticks(LAY_EGG_TICKS) {
                let world = entity.world.load();
                let pos = entity.pos.load();
                world.play_sound(Sound::EntityTurtleLayEgg, SoundCategory::Blocks, &pos);

                let egg_pos = self.move_to_target_pos_goal.target_pos.up();
                let eggs = mob.get_random().random_range(1u8..=4);
                let props = TurtleEggLikeProperties { eggs, hatch: 0 };
                let state_id = props.to_state_id(&Block::TURTLE_EGG);
                world
                    .set_block_state(&egg_pos, state_id, BlockFlags::NOTIFY_ALL)
                    .await;

                turtle.set_has_egg(false);
                turtle.set_laying_egg(false);
                turtle
                    .get_mob_entity()
                    .set_love_ticks(POST_LAY_LOVE_COOLDOWN, None);
            }

            if turtle.is_laying_egg() {
                self.lay_egg_counter += 1;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

impl TurtleLayEggGoal {
    fn is_valid(world: &Arc<World>, block_pos: BlockPos) -> bool {
        if !world.get_block_state(&block_pos.up()).is_air() {
            return false;
        }
        world
            .get_block(&block_pos)
            .has_tag(&tag::Block::MINECRAFT_SAND)
    }
}

impl MoveToTargetPos for TurtleLayEggGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move { Self::is_valid(&world, block_pos) })
    }
}
