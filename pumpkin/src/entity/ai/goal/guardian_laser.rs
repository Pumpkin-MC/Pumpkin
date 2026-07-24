//! Guardian laser beam stand-in (vanilla charges ~80 ticks then damages).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};

const RANGE_SQ: f64 = 12.0 * 12.0;
const CHARGE_TICKS: i32 = 80;
const COOLDOWN: i32 = 40;
/// Vanilla guardian laser damage ~6.
const LASER_DAMAGE: f32 = 6.0;

pub struct GuardianLaserGoal {
    charge: i32,
    speed: f64,
}

impl GuardianLaserGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            charge: 0,
            speed: speed.max(0.2),
        })
    }

    async fn has_los(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let from = mob.get_entity().get_eye_pos();
        let to = target.get_entity().get_eye_pos();
        let world = mob.get_entity().world.load();
        world
            .raycast(from, to, async |block_pos, w| {
                let state = w.get_block_state(block_pos);
                state.is_solid()
            })
            .await
            .is_none()
    }
}

impl Goal for GuardianLaserGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.charge = 0;
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

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
            }

            if dist_sq > RANGE_SQ {
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
                nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
                self.charge = 0;
                return;
            }
            mob.get_mob_entity().navigator.lock().unwrap().stop();

            if !Self::has_los(mob, target.as_ref()).await {
                self.charge = 0;
                return;
            }

            self.charge += 1;
            if self.charge == 1 {
                let world = mob.get_entity().world.load();
                world.play_sound(
                    Sound::EntityGuardianAttack,
                    SoundCategory::Hostile,
                    &mob_pos,
                );
            }

            if self.charge >= to_goal_ticks(CHARGE_TICKS) {
                let _ = target
                    .damage(target.as_ref(), LASER_DAMAGE, DamageType::MAGIC)
                    .await;
                self.charge = -to_goal_ticks(COOLDOWN);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
