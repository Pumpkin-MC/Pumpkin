use pumpkin_util::math::position::BlockPos;

use super::super::OneShot;
use crate::entity::ai::brain::memory::{MemoryModuleId, MemoryStatus, types};

const HIDE_TIMEOUT: i64 = 300;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::HIDING_PLACE.id(), MemoryStatus::ValuePresent),
    (types::HEARD_BELL_TIME.id(), MemoryStatus::ValuePresent),
];

const fn still_hiding(
    ticks_hidden: i32,
    stay_hidden_ticks: i32,
    time_triggered: i64,
    now: i64,
) -> bool {
    ticks_hidden <= stay_hidden_ticks && time_triggered + HIDE_TIMEOUT > now
}

fn closer_than(pos: &BlockPos, other: &BlockPos, distance: i32) -> bool {
    f64::from(pos.squared_distance(other)) < f64::from(distance) * f64::from(distance)
}

#[must_use]
pub fn create(seconds: i32, close_enough_dist: i32) -> OneShot {
    let stay_hidden_ticks = seconds * 20;
    let mut ticks_hidden = 0;
    OneShot::new("SetHiddenState", ENTRY_CONDITIONS.to_vec(), move |tick| {
        let time_triggered = tick
            .state
            .get(types::HEARD_BELL_TIME)
            .copied()
            .unwrap_or_default();
        if still_hiding(ticks_hidden, stay_hidden_ticks, time_triggered, tick.time) {
            if let Some(hiding_place) = tick.state.get(types::HIDING_PLACE) {
                let hide_pos = hiding_place.pos;
                let body_pos = tick.mob.get_entity().block_pos.load();
                if closer_than(&hide_pos, &body_pos, close_enough_dist) {
                    ticks_hidden += 1;
                }
            }
        } else {
            tick.state.erase(types::HEARD_BELL_TIME.id());
            tick.state.erase(types::HIDING_PLACE.id());
            let pos = tick.mob.get_entity().block_pos.load();
            tick.state
                .update_activity_from_schedule(tick.world, tick.time, &pos);
            ticks_hidden = 0;
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn still_hiding_respects_the_stay_and_timeout_boundaries() {
        assert!(still_hiding(0, 20, 100, 100));
        assert!(still_hiding(20, 20, 100, 100));
        assert!(!still_hiding(21, 20, 100, 100));
        assert!(still_hiding(0, 20, 100, 100 + HIDE_TIMEOUT - 1));
        assert!(!still_hiding(0, 20, 100, 100 + HIDE_TIMEOUT));
    }
}
