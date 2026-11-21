use pumpkin_data::environment_attribute::Activity;

use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "ReactToBell",
        vec![(types::HEARD_BELL_TIME.id(), MemoryStatus::ValuePresent)],
        |tick| {
            let pos = tick.mob.get_entity().block_pos.load();
            let has_raid = {
                let raids = tick
                    .world
                    .raids
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                raids.get_raid_at(&pos).is_some()
            };
            if !has_raid {
                tick.state.set_active_activity_if_possible(Activity::Hide);
            }
            true
        },
    )
}
