use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::Block;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const MAX_TIMER: i32 = 40;

pub struct EatGrassGoal {
    goal_control: Controls,
    timer: i32,
}

impl Default for EatGrassGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK | Controls::JUMP,
            timer: 0,
        }
    }
}

impl EatGrassGoal {
    #[must_use]
    pub const fn get_timer(&self) -> i32 {
        self.timer
    }
}

impl Goal for EatGrassGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_random().random_range(0..1000) != 0 {
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let block_pos = entity.block_pos.load();
            let world = entity.world.load();

            let block_at_pos = world.get_block(&block_pos).await;
            if block_at_pos.has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP) {
                return true;
            }

            let block_below = world.get_block(&block_pos.down()).await;
            block_below.id == Block::GRASS_BLOCK.id
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.timer > 0 })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer = MAX_TIMER;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer -= 1;

            if self.timer == 4 {
                let entity = &mob.get_mob_entity().living_entity.entity;
                let block_pos = entity.block_pos.load();
                let world = entity.world.load_full();

                let block_at_pos = world.get_block(&block_pos).await;
                if block_at_pos.has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP) {
                    world
                        .set_block_state(
                            &block_pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    mob.on_eating_grass().await;
                } else {
                    let below_pos = block_pos.down();
                    let block_below = world.get_block(&below_pos).await;
                    if block_below.id == Block::GRASS_BLOCK.id {
                        world
                            .set_block_state(
                                &below_pos,
                                Block::DIRT.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        mob.on_eating_grass().await;
                    }
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer = 0;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
