use pumpkin_data::entity::EntityPose;
use pumpkin_data::environment_attribute::Activity;

use super::super::super::BrainTick;
use super::super::super::memory::types;
use super::super::OneShot;
use crate::entity::passive::villager::VillagerEntity;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("WakeUp", Vec::new(), |tick| {
        if !tick.state.is_active(Activity::Rest) && tick.pose() == EntityPose::Sleeping {
            defer_wake_up(tick);
            true
        } else {
            false
        }
    })
}

pub(super) fn defer_wake_up(tick: &mut BrainTick<'_>) {
    let time = tick.time;
    tick.defer_with_update(
        |mob| {
            if !mob.get_mob_entity().living_entity.is_alive() {
                return false;
            }
            mob.cast_any()
                .downcast_ref::<VillagerEntity>()
                .is_some_and(|villager| {
                    villager.stop_sleeping();
                    true
                })
        },
        move |brain, woke| {
            if woke {
                brain.set(types::LAST_WOKEN, time);
            }
        },
    );
}
