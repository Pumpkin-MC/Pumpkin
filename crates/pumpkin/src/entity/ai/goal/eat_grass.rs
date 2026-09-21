use super::{Controls, Goal};
use crate::entity::ageable::AgeableMob;
use crate::entity::mob::Mob;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const MAX_TIMER: i32 = 40;
const MAX_INTERVAL: i32 = 1000;
const BABY_INTERVAL: i32 = 50;

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
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let interval = self.get_tick_count(if mob.as_ageable().is_some_and(AgeableMob::is_baby) {
            BABY_INTERVAL
        } else {
            MAX_INTERVAL
        });
        if mob.get_random().random_range(0..interval) != 0 {
            return false;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        let block_pos = entity.block_pos.load();
        let world = entity.world.load();

        let block_at_pos = world.get_block(&block_pos);
        if block_at_pos.has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP) {
            return true;
        }

        let block_below = world.get_block(&block_pos.down());
        block_below.id == Block::GRASS_BLOCK.id
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.timer > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.timer = self.get_tick_count(MAX_TIMER);
        let entity = &mob.get_mob_entity().living_entity.entity;
        entity
            .world
            .load()
            .send_entity_status(entity, EntityStatus::EatGrass, None);
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.timer = (self.timer - 1).max(0);

        if self.timer == self.get_tick_count(4) {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let block_pos = entity.block_pos.load();
            let world = entity.world.load_full();

            let mob_griefing = world.level_info.load().game_rules.mob_griefing;

            let block_at_pos = world.get_block(&block_pos);
            if block_at_pos.has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP) {
                if mob_griefing {
                    // Break effects, no drops.
                    world.break_block(&block_pos, None, BlockFlags::SKIP_DROPS);
                }
                mob.on_eating_grass();
            } else {
                let below_pos = block_pos.down();
                let block_below = world.get_block(&below_pos);
                if block_below.id == Block::GRASS_BLOCK.id {
                    if mob_griefing {
                        world.sync_world_event(
                            WorldEvent::ParticlesDestroyBlock,
                            below_pos,
                            i32::from(Block::GRASS_BLOCK.default_state.id.as_u16()),
                        );
                        world.set_block_state(
                            &below_pos,
                            Block::DIRT.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    }
                    mob.on_eating_grass();
                }
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.timer = 0;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
