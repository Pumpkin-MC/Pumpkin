//! Warden sonic boom stand-in (vanilla charges then ranged magic damage).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};

const MIN_RANGE_SQ: f64 = 4.0 * 4.0;
const MAX_RANGE_SQ: f64 = 15.0 * 15.0;
const CHARGE_TICKS: i32 = 34;
const COOLDOWN: i32 = 80;
const BOOM_DAMAGE: f32 = 10.0;

pub struct SonicBoomGoal {
    charge: i32,
}

impl SonicBoomGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { charge: 0 })
    }
}

impl Goal for SonicBoomGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(t) = target.as_ref() else {
                return false;
            };
            if !t.get_entity().is_alive() {
                return false;
            }
            let d = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&t.get_entity().pos.load());
            // Prefer boom when target is not in melee range.
            (MIN_RANGE_SQ..MAX_RANGE_SQ).contains(&d)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| {
                t.get_entity().is_alive()
                    && mob
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&t.get_entity().pos.load())
                        < MAX_RANGE_SQ
            })
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.charge = 0;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            let world = mob.get_entity().world.load();
            world.play_sound(
                Sound::EntityWardenSonicCharge,
                SoundCategory::Hostile,
                &mob.get_entity().pos.load(),
            );
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
            }

            if self.charge < 0 {
                self.charge += 1;
                return;
            }

            self.charge += 1;
            if self.charge >= to_goal_ticks(CHARGE_TICKS) {
                let world = mob.get_entity().world.load();
                world.play_sound(
                    Sound::EntityWardenSonicBoom,
                    SoundCategory::Hostile,
                    &mob.get_entity().pos.load(),
                );
                let _ = target
                    .damage(target.as_ref(), BOOM_DAMAGE, DamageType::MAGIC)
                    .await;
                self.charge = -to_goal_ticks(COOLDOWN);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
