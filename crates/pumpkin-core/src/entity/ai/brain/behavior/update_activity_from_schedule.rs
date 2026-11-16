use super::OneShot;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("UpdateActivityFromSchedule", Vec::new(), |tick| {
        let pos = tick.mob.get_entity().block_pos.load();
        tick.state
            .update_activity_from_schedule(tick.world, tick.time, &pos);
        true
    })
}
