use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use pumpkin_util::math::vector3::Vector3;

const MAX_ATTACK_TIME: i64 = 20;

pub struct MeleeAttackGoal {
    goal_control: Controls,
    speed: f64,
    pause_when_mob_idle: bool,
    //path: Path, TODO: add path when Navigation is implemented
    #[allow(dead_code)]
    target_location: Vector3<f64>,
    update_countdown_ticks: i32,
    pub cooldown: i32,
    #[allow(dead_code)]
    attack_interval_ticks: i32,
    last_update_time: i64,
}

impl MeleeAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            pause_when_mob_idle,
            target_location: Vector3::new(0.0, 0.0, 0.0),
            update_countdown_ticks: 0,
            cooldown: 0,
            attack_interval_ticks: 20,
            last_update_time: 0,
        }
    }

    pub fn get_max_cooldown(&self) -> i32 {
        self.get_tick_count(20)
    }
}

impl Goal for MeleeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let time = {
                let world = &mob.get_entity().world;
                // Assuming world.level_time is the AsyncLevelTimeLock
                let level_time = world.level_time.lock().await;
                level_time.world_age
            };

            if time - self.last_update_time < MAX_ATTACK_TIME {
                return false;
            }
            self.last_update_time = time;

            // 💡 FIX: Await lock operation
            let target = mob.get_mob_entity().target.lock().await;

            let Some(target) = target.as_ref() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }
            // TODO: add path when is implemented Navigation
            true //TODO: modify that because if a path to the target not exists then call mob.is_in_attack_range(target)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;

            let Some(target) = target.as_ref() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }

            if !self.pause_when_mob_idle {
                return !mob.get_mob_entity().navigator.lock().await.is_idle();
            }

            if mob
                .get_mob_entity()
                .is_in_position_target_range_pos(target.get_entity().block_pos.load())
            {
                // This is sync based on the assumed Player methods
                target
                    .get_player()
                    .is_some_and(|player| player.is_spectator() || player.is_creative())
            } else {
                false
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // TODO: add missing fields like mob attacking to true and correct Navigation methods

            if let Some(target) = mob.get_mob_entity().target.lock().await.as_ref() {
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: target.get_entity().pos.load(),
                    speed: self.speed,
                });
            }
            self.update_countdown_ticks = 0;
            self.cooldown = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut target = mob.get_mob_entity().target.lock().await;
            if target.is_none() {
                return;
            }

            if !EntityPredicate::ExceptCreativeOrSpectator
                .test(mob.get_entity())
                .await
            {
                *target = None;
            }

            // TODO: set attacking to false and stop navigation
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // TODO: implement
            // This code is not Vanilla, tick method needs to be reimplemented

            if let Some(target) = mob.get_mob_entity().target.lock().await.as_ref() {
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: target.get_entity().pos.load(),
                    speed: self.speed,
                });
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
