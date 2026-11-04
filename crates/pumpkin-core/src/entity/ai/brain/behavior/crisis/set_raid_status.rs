use pumpkin_data::environment_attribute::Activity;
use rand::RngExt;

use super::super::OneShot;
use crate::world::raid::Raid;

const fn has_first_wave_spawned(raid: &Raid) -> bool {
    raid.get_groups_spawned() > 0
}

fn is_between_waves(raid: &Raid) -> bool {
    has_first_wave_spawned(raid)
        && raid.get_total_raiders_alive() == 0
        && raid.raid_cooldown_ticks > 0
}

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("SetRaidStatus", Vec::new(), |tick| {
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
                .map(|raid| (has_first_wave_spawned(raid), is_between_waves(raid)))
        };
        if let Some((first_wave_spawned, between_waves)) = raid {
            if first_wave_spawned && !between_waves {
                tick.state.set_default_activity(Activity::Raid);
                tick.state.set_active_activity_if_possible(Activity::Raid);
            } else {
                tick.state.set_default_activity(Activity::PreRaid);
                tick.state
                    .set_active_activity_if_possible(Activity::PreRaid);
            }
        }
        true
    })
}
