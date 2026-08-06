use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Simplified approximation of vanilla `Rabbit.customServerAiStep`/`RabbitJumpControl`.
///
/// Arms the physics-level `jumping` flag (the same mechanism `ClimbOnTopOfPowderSnowGoal` uses)
/// whenever the rabbit has an active navigation destination and its jump-delay has elapsed,
/// and re-arms a landing delay once it touches ground again.
///
/// Deliberately does not port vanilla's variable jump-power calculation (`getJumpPower`) or the
/// landing forward-boost (`jumpFromGround`) -- those require a hook into the jump-impulse
/// velocity calculation itself, not just the boolean "should I jump" flag, and are left as a
/// follow-up (see design doc, section 2).
pub struct RabbitHopGoal {
    was_on_ground: bool,
    jump_delay_ticks: i32,
}

impl RabbitHopGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            was_on_ground: true,
            jump_delay_ticks: 0,
        })
    }
}

impl Goal for RabbitHopGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {})
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {})
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let on_ground = entity.on_ground.load(Relaxed);

            if on_ground && !self.was_on_ground {
                // Vanilla `checkLandingDelay`: 10 ticks normally (3 with a >=2.2 speed
                // modifier navigating away from danger; that faster case isn't distinguished
                // here, see the module doc comment).
                self.jump_delay_ticks = 10;
                mob_entity.living_entity.jumping.store(false, SeqCst);
            }
            self.was_on_ground = on_ground;

            if self.jump_delay_ticks > 0 {
                self.jump_delay_ticks -= 1;
                return;
            }

            if !on_ground {
                return;
            }

            let has_destination = !mob_entity.navigator.lock().unwrap().is_idle();
            if has_destination {
                mob_entity.living_entity.jumping.store(true, SeqCst);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}
