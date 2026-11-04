use pumpkin_data::entity::EntityPose;
use pumpkin_data::environment_attribute::Activity;

use super::super::OneShot;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("WakeUp", Vec::new(), |tick| {
        !tick.state.is_active(Activity::Rest)
            && tick.mob.get_entity().pose.load() == EntityPose::Sleeping
    })
}
