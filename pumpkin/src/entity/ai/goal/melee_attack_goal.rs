use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use async_trait::async_trait;
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

#[async_trait]
impl Goal for MeleeAttackGoal {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let time = {
            let world = &mob.get_entity().world;
            let level_time = world.level_time.lock().await;
            level_time.world_age
        };

        if time - self.last_update_time < MAX_ATTACK_TIME {
            return false;
        }
        self.last_update_time = time;
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

    async fn start(&mut self, mob: &dyn Mob) {
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
    }

    async fn stop(&mut self, mob: &dyn Mob) {
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

    async fn tick(&mut self, mob: &dyn Mob) {
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

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
