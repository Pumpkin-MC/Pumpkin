//! Goat ram charge (vanilla prepare-ram + impact simplified).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};

const RAM_RANGE_SQ: f64 = 12.0 * 12.0;
const MIN_RANGE_SQ: f64 = 2.0 * 2.0;
const CHARGE_TICKS: i32 = 20;
const COOLDOWN: i32 = 60;
const RAM_DAMAGE: f32 = 4.0;
const RAM_SPEED: f64 = 1.8;

pub struct RamGoal {
    /// >0 charging, 0 ready, <0 cooldown.
    phase: i32,
}

impl RamGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { phase: 0 })
    }
}

impl Goal for RamGoal {
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
            (MIN_RANGE_SQ..RAM_RANGE_SQ).contains(&d)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.phase = 1;
            let world = mob.get_entity().world.load();
            world.play_sound(
                Sound::EntityGoatPrepareRam,
                SoundCategory::Neutral,
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

            if self.phase < 0 {
                self.phase += 1;
                return;
            }

            let mob_pos = mob.get_entity().pos.load();
            let tpos = target.get_entity().pos.load();
            let dx = tpos.x - mob_pos.x;
            let dz = tpos.z - mob_pos.z;
            let len = (dx * dx + dz * dz).sqrt().max(0.001);

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 40.0, 40.0);
            }

            // Dash toward target.
            let mut vel = mob.get_entity().velocity.load();
            vel.x = (dx / len) * RAM_SPEED * 0.35;
            vel.z = (dz / len) * RAM_SPEED * 0.35;
            mob.get_entity().set_velocity(vel);

            self.phase += 1;
            let dist_sq = mob_pos.squared_distance_to_vec(&tpos);
            if dist_sq < MIN_RANGE_SQ || self.phase >= to_goal_ticks(CHARGE_TICKS) {
                let world = mob.get_entity().world.load();
                world.play_sound(
                    Sound::EntityGoatRamImpact,
                    SoundCategory::Neutral,
                    &mob_pos,
                );
                let _ = target
                    .damage(target.as_ref(), RAM_DAMAGE, DamageType::MOB_ATTACK)
                    .await;
                // Knock target back.
                let mut tvel = target.get_entity().velocity.load();
                tvel.x += (dx / len) * 0.6;
                tvel.y += 0.25;
                tvel.z += (dz / len) * 0.6;
                target.get_entity().set_velocity(tvel);
                target.get_entity().send_velocity();
                self.phase = -to_goal_ticks(COOLDOWN);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
