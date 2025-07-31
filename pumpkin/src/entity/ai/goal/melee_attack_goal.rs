use super::{Control, Goal, GoalControl};
use crate::entity::EntityBase;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicI64};

const MAX_ATTACK_TIME: i64 = 20;

pub struct MeleeAttackGoal {
    goal_control: GoalControl,
    speed: f64,
    pause_when_mob_idle: bool,
    //path: Path, TODO: add path when Navigation is implemented
    #[allow(dead_code)]
    target_location: AtomicCell<Vector3<f64>>,
    update_countdown_ticks: AtomicI32,
    pub cooldown: AtomicI32,
    #[allow(dead_code)]
    attack_interval_ticks: i32,
    last_update_time: AtomicI64,
}

impl MeleeAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            goal_control: GoalControl::from_array(&[Control::Move, Control::Look]),
            speed,
            pause_when_mob_idle,
            target_location: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            update_countdown_ticks: AtomicI32::new(0),
            cooldown: AtomicI32::new(0),
            attack_interval_ticks: 20,
            last_update_time: AtomicI64::new(0),
        }
    }

    pub fn get_max_cooldown(&self) -> i32 {
        self.get_tick_count(20)
    }
}

#[async_trait]
impl Goal for MeleeAttackGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        let time = {
            let world = mob.get_entity().world.read().await;
            let level_time = world.level_time.lock().await;
            level_time.world_age
        };

        if time - self.last_update_time.load(Relaxed) < MAX_ATTACK_TIME {
            return false;
        }
        self.last_update_time.store(time, Relaxed);
        let target = mob.get_mob_entity().target.lock().await;
        let Some(target) = target.as_ref() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }
        // TODO: add path when is implemented Navigation
        true //TODO: modify that because if a path to the target not exists then call mob.is_in_attack_range(target)
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
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
            target
                .get_player()
                .is_some_and(|player| player.is_spectator() || player.is_creative())
        } else {
            false
        }
    }

    async fn start(&self, mob: &dyn Mob) {
        // TODO: add missing fields like mob attacking to true and correct Navigation methods
        if let Some(target) = mob.get_mob_entity().target.lock().await.as_ref() {
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.set_progress(NavigatorGoal {
                current_progress: mob.get_entity().pos.load(),
                destination: target.get_entity().pos.load(),
                speed: self.speed,
            });
        }
        self.update_countdown_ticks.store(0, Relaxed);
        self.cooldown.store(0, Relaxed);
    }

    async fn stop(&self, mob: &dyn Mob) {
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
    }

    async fn tick(&self, mob: &dyn Mob) {
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
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}
