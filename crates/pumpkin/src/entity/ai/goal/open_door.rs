use super::door_interact::DoorInteractGoal;
use super::{Controls, Goal};
use crate::entity::mob::Mob;

pub struct OpenDoorGoal {
    pub door_interact_goal: DoorInteractGoal,
    pub close_door: bool,
    pub forget_time: i32,
}

impl OpenDoorGoal {
    #[must_use]
    pub const fn new(close_door_after: bool) -> Self {
        Self {
            door_interact_goal: DoorInteractGoal::new(),
            close_door: close_door_after,
            forget_time: 0,
        }
    }
}

impl Default for OpenDoorGoal {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Goal for OpenDoorGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.door_interact_goal.can_use(mob)
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.close_door && self.forget_time > 0 && self.door_interact_goal.can_continue_to_use()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.door_interact_goal.start_interaction(mob);
        self.forget_time = 20;
        self.door_interact_goal.set_open(mob, true);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if self.close_door {
            self.door_interact_goal.set_open(mob, false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.forget_time -= 1;
        self.door_interact_goal.tick_interaction(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
