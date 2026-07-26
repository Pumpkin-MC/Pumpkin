//! Vanilla `DoorInteractGoal` family — mobs that walk into a door either open
//! it or beat it down instead of pushing against it forever.

use std::sync::atomic::Ordering;

use pumpkin_data::world::WorldEvent;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use super::{Goal, GoalFuture};
use crate::block::blocks::doors::{is_door_open, is_mob_interactable_door, set_door_open};
use crate::entity::mob::Mob;

/// Shared state of vanilla's abstract `DoorInteractGoal`.
///
/// Tracks the door the mob is standing in front of and the direction it faced
/// when the goal started, so the goal can tell once the mob has walked through.
pub struct DoorInteractState {
    door_pos: BlockPos,
    has_door: bool,
    passed: bool,
    door_open_dir_x: f32,
    door_open_dir_z: f32,
}

impl Default for DoorInteractState {
    fn default() -> Self {
        Self {
            door_pos: BlockPos::ZERO,
            has_door: false,
            passed: false,
            door_open_dir_x: 0.0,
            door_open_dir_z: 0.0,
        }
    }
}

impl DoorInteractState {
    /// Vanilla `DoorInteractGoal.canUse` — look for a door on the next few path
    /// nodes, falling back to the block the mob's head is in.
    fn find_door(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        if !entity.horizontal_collision.load(Ordering::Relaxed) {
            return false;
        }

        let world = entity.world.load();
        let pos = entity.pos.load();

        // Scope the navigator lock: nothing here awaits.
        let path_door = {
            let navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.current_path().and_then(|path| {
                if path.is_done() {
                    return None;
                }
                let limit = (path.get_next_node_index() + 2).min(path.get_node_count());
                (0..limit).find_map(|index| {
                    let node = path.get_node(index)?;
                    let door_pos = BlockPos::new(node.pos.0.x, node.pos.0.y + 1, node.pos.0.z);
                    let dx = f64::from(door_pos.0.x) - pos.x;
                    let dz = f64::from(door_pos.0.z) - pos.z;
                    if dx.mul_add(dx, dz * dz) > 2.25 {
                        return None;
                    }
                    is_mob_interactable_door(world.get_block(&door_pos)).then_some(door_pos)
                })
            })
        };

        if let Some(door_pos) = path_door {
            self.door_pos = door_pos;
            self.has_door = true;
            return true;
        }

        // Vanilla falls back to the block occupied by the mob's head.
        let door_pos = entity.block_pos.load().up();
        self.has_door = is_mob_interactable_door(world.get_block(&door_pos));
        self.door_pos = door_pos;
        self.has_door
    }

    /// Vanilla `DoorInteractGoal.isOpen`. Clears `has_door` when the block is
    /// no longer a door, matching vanilla's self-invalidation.
    fn is_open(&mut self, mob: &dyn Mob) -> bool {
        if !self.has_door {
            return false;
        }
        let world = mob.get_entity().world.load();
        let open = is_door_open(&world, &self.door_pos);
        if open.is_none() {
            self.has_door = false;
        }
        open.unwrap_or(false)
    }

    async fn set_open(&self, mob: &dyn Mob, open: bool) {
        if !self.has_door {
            return;
        }
        let world = mob.get_entity().world.load_full();
        set_door_open(&world, &self.door_pos, open).await;
    }

    fn start(&mut self, mob: &dyn Mob) {
        let pos = mob.get_entity().pos.load();
        self.passed = false;
        self.door_open_dir_x = (f64::from(self.door_pos.0.x) + 0.5 - pos.x) as f32;
        self.door_open_dir_z = (f64::from(self.door_pos.0.z) + 0.5 - pos.z) as f32;
    }

    /// Vanilla `DoorInteractGoal.tick` — the dot product flips sign once the mob
    /// has walked past the door.
    fn tick(&mut self, mob: &dyn Mob) {
        let pos = mob.get_entity().pos.load();
        let new_dir_x = (f64::from(self.door_pos.0.x) + 0.5 - pos.x) as f32;
        let new_dir_z = (f64::from(self.door_pos.0.z) + 0.5 - pos.z) as f32;
        if self
            .door_open_dir_x
            .mul_add(new_dir_x, self.door_open_dir_z * new_dir_z)
            < 0.0
        {
            self.passed = true;
        }
    }
}

/// Vanilla `BreakDoorGoal` — zombies (and raiding vindicators) smash through
/// wooden doors when the difficulty allows it.
pub struct BreakDoorGoal {
    state: DoorInteractState,
    /// Difficulties at which the door may be broken (vanilla `validDifficulties`).
    valid_difficulty: fn(Difficulty) -> bool,
    break_time: i32,
    last_break_progress: i32,
    door_break_time: i32,
}

impl BreakDoorGoal {
    const DEFAULT_DOOR_BREAK_TIME: i32 = 240;

    #[must_use]
    pub fn new(valid_difficulty: fn(Difficulty) -> bool) -> Self {
        Self {
            state: DoorInteractState::default(),
            valid_difficulty,
            break_time: 0,
            last_break_progress: -1,
            door_break_time: -1,
        }
    }

    /// Vanilla constructor taking an explicit break duration in ticks.
    #[must_use]
    pub fn with_break_time(valid_difficulty: fn(Difficulty) -> bool, ticks: i32) -> Self {
        let mut goal = Self::new(valid_difficulty);
        goal.door_break_time = ticks;
        goal
    }

    fn door_break_time(&self) -> i32 {
        self.door_break_time.max(Self::DEFAULT_DOOR_BREAK_TIME)
    }

    fn difficulty_allows(&self, mob: &dyn Mob) -> bool {
        let difficulty = mob.get_entity().world.load().level_info.load().difficulty;
        (self.valid_difficulty)(difficulty)
    }
}

impl Goal for BreakDoorGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !mob.can_break_doors() {
                return false;
            }
            let world = mob.get_entity().world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }
            drop(world);
            self.state.find_door(mob) && self.difficulty_allows(mob) && !self.state.is_open(mob)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.break_time > self.door_break_time() || !self.difficulty_allows(mob) {
                return false;
            }
            if !self.state.has_door {
                return false;
            }
            let world = mob.get_entity().world.load();
            if is_door_open(&world, &self.state.door_pos).unwrap_or(true) {
                return false;
            }
            // Vanilla `doorPos.closerToCenterThan(mob.position(), 2.0)`.
            let pos = mob.get_entity().pos.load();
            let dx = f64::from(self.state.door_pos.0.x) + 0.5 - pos.x;
            let dy = f64::from(self.state.door_pos.0.y) + 0.5 - pos.y;
            let dz = f64::from(self.state.door_pos.0.z) + 0.5 - pos.z;
            dx.mul_add(dx, dy.mul_add(dy, dz * dz)) < 4.0
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.state.start(mob);
            self.break_time = 0;
            self.last_break_progress = -1;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let world = mob.get_entity().world.load_full();
            world
                .set_block_breaking(mob.get_entity(), self.state.door_pos, -1)
                .await;
            self.last_break_progress = -1;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.state.tick(mob);
            let world = mob.get_entity().world.load_full();
            let door_pos = self.state.door_pos;

            if mob.get_random().random_range(0..20) == 0 {
                world.sync_world_event(WorldEvent::SoundZombieWoodenDoor, door_pos, 0);
                mob.get_mob_entity().living_entity.swing_hand().await;
            }

            self.break_time += 1;
            let break_time = self.door_break_time();
            let progress = (self.break_time as f32 / break_time as f32 * 10.0) as i32;
            if progress != self.last_break_progress {
                world
                    .set_block_breaking(mob.get_entity(), door_pos, progress)
                    .await;
                self.last_break_progress = progress;
            }

            if self.break_time == break_time && self.difficulty_allows(mob) {
                world
                    .break_block(
                        &door_pos,
                        None,
                        BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                world.sync_world_event(WorldEvent::SoundZombieDoorCrash, door_pos, 0);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }
}

/// Vanilla `OpenDoorGoal` — a mob pushes a door open on its way through and
/// optionally pulls it shut behind itself.
pub struct OpenDoorGoal {
    state: DoorInteractState,
    close_door: bool,
    forget_time: i32,
}

impl OpenDoorGoal {
    #[must_use]
    pub fn new(close_door_after: bool) -> Self {
        Self {
            state: DoorInteractState::default(),
            close_door: close_door_after,
            forget_time: 0,
        }
    }
}

impl Goal for OpenDoorGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.state.find_door(mob) })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.close_door && self.forget_time > 0 && !self.state.passed })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.state.start(mob);
            self.forget_time = 20;
            self.state.set_open(mob, true).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.state.set_open(mob, false).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.forget_time -= 1;
            self.state.tick(mob);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }
}
