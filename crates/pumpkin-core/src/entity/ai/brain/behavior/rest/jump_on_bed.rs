use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget};
use super::super::timed::Behavior;

const MAX_TIME_TO_REACH_BED: i32 = 100;
const MIN_JUMPS: i32 = 3;
const MAX_JUMPS: i32 = 6;
const COOLDOWN_BETWEEN_JUMPS: i32 = 5;
const NO_TIMEOUT: i32 = i32::MAX / 2;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::NEAREST_BED.id(), MemoryStatus::ValuePresent),
    (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
];

pub struct JumpOnBed {
    speed_modifier: f32,
    target_bed: Option<BlockPos>,
    remaining_time_to_reach_bed: i32,
    remaining_jumps: i32,
    remaining_cooldown_until_next_jump: i32,
}

impl JumpOnBed {
    #[must_use]
    pub const fn new(speed_modifier: f32) -> Self {
        Self {
            speed_modifier,
            target_bed: None,
            remaining_time_to_reach_bed: 0,
            remaining_jumps: 0,
            remaining_cooldown_until_next_jump: 0,
        }
    }

    fn start_walking_towards_bed(&self, tick: &mut BrainTick<'_>, bed_pos: BlockPos) {
        tick.state.set(
            types::WALK_TARGET,
            WalkTarget::from_block_pos(bed_pos, self.speed_modifier, 0),
        );
    }
}

impl Behavior for JumpOnBed {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn max_duration(&self) -> i32 {
        NO_TIMEOUT
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_mob_entity().living_entity.is_baby() && near_bed(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(target_bed) = get_nearest_bed(tick) {
            self.target_bed = Some(target_bed);
            self.remaining_time_to_reach_bed = MAX_TIME_TO_REACH_BED;
            self.remaining_jumps = initial_remaining_jumps(
                tick.mob
                    .get_random()
                    .random_range(0..(MAX_JUMPS - MIN_JUMPS + 1)),
            );
            self.remaining_cooldown_until_next_jump = 0;
            self.start_walking_towards_bed(tick, target_bed);
        }
    }

    fn stop(&mut self, _tick: &mut BrainTick<'_>) {
        self.target_bed = None;
        self.remaining_time_to_reach_bed = 0;
        self.remaining_jumps = 0;
        self.remaining_cooldown_until_next_jump = 0;
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !tick.mob.get_mob_entity().living_entity.is_baby() {
            return false;
        }
        let Some(target_bed) = self.target_bed else {
            return false;
        };
        if !is_jumpable(tick, target_bed) {
            return false;
        }
        let on_or_over = on_or_over_bed(tick);
        !tired_of_walking(on_or_over, self.remaining_time_to_reach_bed)
            && !tired_of_jumping(on_or_over, self.remaining_jumps)
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if !on_or_over_bed(tick) {
            self.remaining_time_to_reach_bed -= 1;
        } else if self.remaining_cooldown_until_next_jump > 0 {
            self.remaining_cooldown_until_next_jump -= 1;
        } else if on_bed_surface(tick) {
            tick.mob
                .get_mob_entity()
                .jump_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .jump();
            self.remaining_jumps = jumps_after_jump(self.remaining_jumps);
            self.remaining_cooldown_until_next_jump = COOLDOWN_BETWEEN_JUMPS;
        }
    }

    fn debug_name(&self) -> &'static str {
        "JumpOnBed"
    }
}

const fn initial_remaining_jumps(random_offset: i32) -> i32 {
    MIN_JUMPS + random_offset
}

const fn jumps_after_jump(remaining_jumps: i32) -> i32 {
    remaining_jumps - 1
}

const fn tired_of_walking(on_or_over_bed: bool, remaining_time_to_reach_bed: i32) -> bool {
    !on_or_over_bed && remaining_time_to_reach_bed <= 0
}

const fn tired_of_jumping(on_or_over_bed: bool, remaining_jumps: i32) -> bool {
    on_or_over_bed && remaining_jumps <= 0
}

fn near_bed(tick: &BrainTick<'_>) -> bool {
    on_or_over_bed(tick) || get_nearest_bed(tick).is_some()
}

fn on_or_over_bed(tick: &BrainTick<'_>) -> bool {
    let body_pos = tick.mob.get_entity().block_pos.load();
    is_jumpable(tick, body_pos) || is_jumpable(tick, body_pos.down())
}

fn on_bed_surface(tick: &BrainTick<'_>) -> bool {
    is_jumpable(tick, tick.mob.get_entity().block_pos.load())
}

fn is_jumpable(tick: &BrainTick<'_>, pos: BlockPos) -> bool {
    tick.world
        .get_block_and_state(&pos)
        .0
        .has_tag(&BlockTag::MINECRAFT_VILLAGER_BABIES_CAN_JUMP_ON_BED)
}

fn get_nearest_bed(tick: &BrainTick<'_>) -> Option<BlockPos> {
    tick.state.get(types::NEAREST_BED).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tired_of_walking_requires_no_bed_and_an_expired_timer() {
        assert!(!tired_of_walking(true, 0));
        assert!(!tired_of_walking(true, -1));
        assert!(!tired_of_walking(false, 1));
        assert!(tired_of_walking(false, 0));
        assert!(tired_of_walking(false, -1));
    }

    #[test]
    fn tired_of_jumping_requires_a_bed_and_no_jumps_left() {
        assert!(!tired_of_jumping(false, 0));
        assert!(!tired_of_jumping(false, -1));
        assert!(!tired_of_jumping(true, 1));
        assert!(tired_of_jumping(true, 0));
        assert!(tired_of_jumping(true, -1));
    }

    #[test]
    fn jump_counter_starts_between_min_and_max_and_counts_down() {
        assert_eq!(initial_remaining_jumps(0), MIN_JUMPS);
        assert_eq!(initial_remaining_jumps(MAX_JUMPS - MIN_JUMPS), MAX_JUMPS);
        assert_eq!(jumps_after_jump(MIN_JUMPS), MIN_JUMPS - 1);
        assert_eq!(jumps_after_jump(1), 0);
    }
}
