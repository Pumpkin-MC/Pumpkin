use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::Difficulty;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use super::door_interact::DoorInteractGoal;
use super::{Controls, Goal};
use crate::block::blocks::doors::DoorBlock;
use crate::entity::mob::Mob;

const DEFAULT_DOOR_BREAK_TIME: i32 = 240;

pub type DifficultyPredicate = Arc<dyn Fn(Difficulty) -> bool + Send + Sync>;

pub struct BreakDoorGoal {
    pub door_interact_goal: DoorInteractGoal,
    valid_difficulties: DifficultyPredicate,
    pub break_time: i32,
    pub last_break_progress: i32,
    pub door_break_time: i32,
}

impl BreakDoorGoal {
    #[must_use]
    pub fn new(valid_difficulties: DifficultyPredicate) -> Self {
        Self {
            door_interact_goal: DoorInteractGoal::new(),
            valid_difficulties,
            break_time: 0,
            last_break_progress: -1,
            door_break_time: -1,
        }
    }

    #[must_use]
    pub fn with_door_break_time(
        door_break_time: i32,
        valid_difficulties: DifficultyPredicate,
    ) -> Self {
        Self {
            door_interact_goal: DoorInteractGoal::new(),
            valid_difficulties,
            break_time: 0,
            last_break_progress: -1,
            door_break_time,
        }
    }

    #[must_use]
    pub const fn get_door_break_time(&self) -> i32 {
        if self.door_break_time > DEFAULT_DOOR_BREAK_TIME {
            self.door_break_time
        } else {
            DEFAULT_DOOR_BREAK_TIME
        }
    }

    #[must_use]
    pub fn is_valid_difficulty(&self, difficulty: Difficulty) -> bool {
        (self.valid_difficulties)(difficulty)
    }
}

impl Default for BreakDoorGoal {
    fn default() -> Self {
        Self::new(Arc::new(|d| d == Difficulty::Hard))
    }
}

impl Goal for BreakDoorGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !self.door_interact_goal.can_use(mob) {
            return false;
        }
        let world = mob.get_entity().world.load();
        let level_info = world.level_info.load();
        if !level_info.game_rules.mob_griefing {
            return false;
        }
        self.is_valid_difficulty(level_info.difficulty) && !self.door_interact_goal.is_open(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        let level_info = world.level_info.load();
        let mob_pos = mob.get_entity().pos.load();
        let door_pos = self.door_interact_goal.door_pos;
        let center_x = f64::from(door_pos.0.x) + 0.5;
        let center_y = f64::from(door_pos.0.y) + 0.5;
        let center_z = f64::from(door_pos.0.z) + 0.5;
        let dx = center_x - mob_pos.x;
        let dy = center_y - mob_pos.y;
        let dz = center_z - mob_pos.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;

        self.break_time <= self.get_door_break_time()
            && !DoorBlock::is_open(&world, &door_pos)
            && dist_sq < 4.0
            && self.is_valid_difficulty(level_info.difficulty)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.door_interact_goal.start_interaction(mob);
        self.break_time = 0;
        self.last_break_progress = -1;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let world = mob.get_entity().world.load();
        world.set_block_destroy_stage(
            mob.get_entity().entity_id,
            self.door_interact_goal.door_pos,
            -1,
        );
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.door_interact_goal.tick_interaction(mob);
        let world = mob.get_entity().world.load_full();

        if mob.get_random().random_range(0..20) == 0 {
            world.sync_world_event(
                WorldEvent::SoundZombieWoodenDoor,
                self.door_interact_goal.door_pos,
                0,
            );
            mob.get_mob_entity().living_entity.swing_hand();
        }

        self.break_time += 1;
        let progress = (self.break_time as f32 / self.get_door_break_time() as f32 * 10.0) as i32;
        if progress != self.last_break_progress {
            world.set_block_destroy_stage(
                mob.get_entity().entity_id,
                self.door_interact_goal.door_pos,
                progress as i8,
            );
            self.last_break_progress = progress;
        }

        let level_info = world.level_info.load();
        if self.break_time == self.get_door_break_time()
            && self.is_valid_difficulty(level_info.difficulty)
        {
            let (_, block_state_id) =
                world.get_block_and_state_id(&self.door_interact_goal.door_pos);
            let door_pos = self.door_interact_goal.door_pos;
            let mut event =
                crate::plugin::api::events::entity::entity_break_door::EntityBreakDoorEvent::new(
                    mob.get_entity().entity_id,
                    door_pos,
                );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            world.set_block_state(&door_pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
            world.sync_world_event(
                WorldEvent::SoundZombieDoorCrash,
                self.door_interact_goal.door_pos,
                0,
            );
            world.sync_world_event(
                WorldEvent::ParticlesDestroyBlock,
                self.door_interact_goal.door_pos,
                i32::from(block_state_id.as_u16()),
            );
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
