use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct RandomStandGoal {
    next_stand: i32,
}

impl RandomStandGoal {
    #[must_use]
    pub const fn new(ambient_stand_interval: i32) -> Self {
        Self {
            next_stand: -ambient_stand_interval,
        }
    }
}

impl Goal for RandomStandGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.next_stand += 1;
        if self.next_stand <= 0 || mob.get_random().random_range(0..1000) >= self.next_stand {
            return false;
        }
        self.next_stand = -mob.get_ambient_sound_interval();
        !mob.is_immobile() && mob.get_random().random_range(0..10) == 0
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(horse) = mob.as_abstract_horse() else {
            return;
        };
        horse.stand_if_possible();
        if let Some(sound) = mob.get_ambient_sound() {
            mob.play_entity_sound(sound, 1.0, 1.0);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
