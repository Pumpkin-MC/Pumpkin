use super::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::entity::mob::sun_burn;

#[derive(Default)]
pub struct RestrictSunGoal;

impl RestrictSunGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn is_exposed(mob: &dyn Mob) -> bool {
        mob.get_entity().world.load().is_bright_outside() && !sun_burn::is_protected(mob)
    }
}

impl Goal for RestrictSunGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::is_exposed(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        Self::is_exposed(mob)
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
