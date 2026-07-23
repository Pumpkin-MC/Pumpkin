use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Holds a tameable mob in place while ordered to sit.
///
/// Occupies MOVE|JUMP so wander / follow / attack goals cannot run until
/// `is_sitting()` becomes false.
pub struct SitGoal;

impl SitGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Default for SitGoal {
    fn default() -> Self {
        Self
    }
}

impl Goal for SitGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { mob.is_sitting() && mob.get_owner_uuid().is_some() })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { mob.is_sitting() && mob.get_owner_uuid().is_some() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Keep navigator idle while sitting so repath does not resume movement.
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            if !navigator.is_idle() {
                navigator.stop();
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}
