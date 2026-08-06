use std::sync::Weak;

use pumpkin_data::sound::{Sound, SoundCategory};
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::vex::VexEntity;

/// Vanilla: `Vex.VexChargeAttackGoal`.
///
/// Charges the move-control straight at the target's eye position; if the vex's bounding box
/// reaches the target it attacks directly, otherwise it keeps re-aiming while within
/// `distanceToSqr < 9.0` (mid-charge homing correction).
pub struct VexChargeAttackGoal {
    vex: Weak<VexEntity>,
}

impl VexChargeAttackGoal {
    #[must_use]
    pub const fn new(vex: Weak<VexEntity>) -> Self {
        Self { vex }
    }
}

impl Goal for VexChargeAttackGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return false;
            };
            let Some(target) = vex.mob_entity.target.lock().await.clone() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }
            if vex.mob_entity.move_control.lock().unwrap().has_wanted() {
                return false;
            }
            if rand::rng().random_range(0..to_goal_ticks(7)) != 0 {
                return false;
            }
            let vex_pos = vex.mob_entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            vex_pos.squared_distance_to_vec(&target_pos) > 4.0
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return false;
            };
            if !vex.mob_entity.move_control.lock().unwrap().has_wanted() || !vex.is_charging() {
                return false;
            }
            vex.mob_entity
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return;
            };
            if let Some(target) = vex.mob_entity.target.lock().await.as_ref() {
                let eye_pos = target.get_entity().get_eye_pos();
                vex.mob_entity
                    .move_control
                    .lock()
                    .unwrap()
                    .set_wanted_position(eye_pos.x, eye_pos.y, eye_pos.z, 1.0);
            }
            vex.set_is_charging(true);
            vex.mob_entity.living_entity.entity.world.load().play_sound(
                Sound::EntityVexCharge,
                SoundCategory::Hostile,
                &vex.mob_entity.living_entity.entity.pos.load(),
            );
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(vex) = self.vex.upgrade() {
                vex.set_is_charging(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return;
            };
            let Some(target) = vex.mob_entity.target.lock().await.clone() else {
                return;
            };
            let vex_box = vex.mob_entity.living_entity.entity.bounding_box.load();
            let target_box = target.get_entity().bounding_box.load();
            if vex_box.intersects(&target_box) {
                mob.try_attack(target.as_ref()).await;
                vex.set_is_charging(false);
                return;
            }
            let vex_pos = vex.mob_entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if vex_pos.squared_distance_to_vec(&target_pos) < 9.0 {
                let eye_pos = target.get_entity().get_eye_pos();
                vex.mob_entity
                    .move_control
                    .lock()
                    .unwrap()
                    .set_wanted_position(eye_pos.x, eye_pos.y, eye_pos.z, 1.0);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
