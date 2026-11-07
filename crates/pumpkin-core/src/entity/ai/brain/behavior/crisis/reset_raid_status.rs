use pumpkin_data::environment_attribute::Activity;
use rand::RngExt;

use super::super::OneShot;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("ResetRaidStatus", Vec::new(), |tick| {
        if tick.mob.get_random().random_range(0..20) != 0 {
            return false;
        }
        let pos = tick.mob.get_entity().block_pos.load();
        let raid = {
            let raids = tick
                .world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            raids
                .get_raid_at(&pos)
                .map(|raid| (raid.is_stopped(), raid.is_loss()))
        };
        if raid.is_none_or(|(stopped, loss)| stopped || loss) {
            tick.state.set_default_activity(Activity::Idle);
            tick.state
                .update_activity_from_schedule(tick.world, tick.time, &pos);
        }
        true
    })
}
