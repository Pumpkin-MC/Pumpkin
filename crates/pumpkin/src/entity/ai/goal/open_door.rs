use super::door_interact::DoorInteractGoal;
use super::{Controls, Goal, GoalFuture};
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
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.door_interact_goal.can_use(mob) })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.close_door && self.forget_time > 0 && self.door_interact_goal.can_continue_to_use()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.door_interact_goal.start_interaction(mob);
            self.forget_time = 20;
            self.door_interact_goal.set_open(mob, true).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if self.close_door {
                self.door_interact_goal.set_open(mob, false).await;
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.forget_time -= 1;
            self.door_interact_goal.tick_interaction(mob);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_door_can_continue_to_use() {
        let mut goal = OpenDoorGoal::new(true);
        goal.forget_time = 20;
        goal.door_interact_goal.passed = false;

        assert!(goal.close_door);
        assert!(goal.forget_time > 0);
        assert!(goal.door_interact_goal.can_continue_to_use());

        // When forget_time expires
        goal.forget_time = 0;
        assert!(
            !(goal.close_door
                && goal.forget_time > 0
                && goal.door_interact_goal.can_continue_to_use())
        );

        // When close_door is false
        let goal_no_close = OpenDoorGoal::new(false);
        assert!(!goal_no_close.close_door);
    }
}
