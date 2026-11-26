use pumpkin_data::environment_attribute::Activity;

use super::super::super::memory::types;
use super::super::OneShot;
use crate::entity::passive::villager::VillagerEntity;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("WakeUp", Vec::new(), |tick| {
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return false;
        };
        if !tick.state.is_active(Activity::Rest) && villager.is_sleeping() {
            villager.stop_sleeping();
            tick.state.set(types::LAST_WOKEN, tick.time);
            true
        } else {
            false
        }
    })
}
