use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const MAX_ATTACK_TIME: i64 = 20;

pub struct MeleeAttackGoal {
    goal_control: Controls,
    speed: f64,
    pause_when_mob_idle: bool,
    //path: Path, TODO: add path when Navigation is implemented
    #[expect(dead_code)]
    target_location: Vector3<f64>,
    update_countdown_ticks: i32,
    pub cooldown: i32,
    #[expect(dead_code)]
    attack_interval_ticks: i32,
    last_update_time: i64,
    last_target_position: Option<Vector3<f64>>,
}

impl MeleeAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed: speed.max(0.23), // Ensure minimum visible speed
            pause_when_mob_idle,
            target_location: Vector3::new(0.0, 0.0, 0.0),
            update_countdown_ticks: 0,
            cooldown: 0,
            attack_interval_ticks: 20,
            last_update_time: 0,
            last_target_position: None,
        }
    }

    #[must_use]
    pub fn get_max_cooldown(&self) -> i32 {
        self.get_tick_count(20)
    }
}

impl Goal for MeleeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let time = {
                let world = mob.get_entity().world.load();
                // Assuming world.level_time is the AsyncLevelTimeLock
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

            let is_valid_target = !target
                .get_player()
                .is_some_and(|p| p.is_spectator() || p.is_creative());

            let in_range = mob
                .get_mob_entity()
                .is_in_position_target_range_pos(&target.get_entity().block_pos.load());

            in_range && is_valid_target
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // TODO: add missing fields like mob attacking to true and correct Navigation methods

            if let Some(target) = mob.get_mob_entity().target.lock().await.as_ref() {
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                let target_pos = target.get_entity().pos.load();
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: target_pos,
                    speed: self.speed,
                });
                self.last_target_position = Some(target_pos);
            }
            self.update_countdown_ticks = 0;
            self.cooldown = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mut target = mob.get_mob_entity().target.lock().await;

            if let Some(entity) = target.as_deref()
                && !EntityPredicate::ExceptCreativeOrSpectator
                    .test(entity.get_entity())
                    .await
            {
                *target = None;
            }

            self.last_target_position = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target_lock = mob.get_mob_entity().target.lock().await;
            let Some(target) = target_lock.as_ref() else {
                return;
            };

            mob.get_mob_entity()
                .look_control
                .lock()
                .await
                .look_at_entity_with_range(target, 30.0, 30.0);

            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);

            let current_target_pos = target.get_entity().pos.load();
            let should_update_nav = self.update_countdown_ticks <= 0
                && (self.last_target_position.is_none_or(|last_pos| {
                    current_target_pos.squared_distance_to_vec(&last_pos) >= 1.0
                }) || mob.get_random().random_range(0..20) == 0);

            if should_update_nav {
                let mob_pos = mob.get_entity().pos.load();
                let dist_sq = mob_pos.squared_distance_to_vec(&current_target_pos);
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: current_target_pos,
                    speed: self.speed,
                });
                self.last_target_position = Some(current_target_pos);
                self.update_countdown_ticks = 4 + mob.get_random().random_range(0..7);
                if dist_sq > 1024.0 {
                    self.update_countdown_ticks += 10;
                } else if dist_sq > 256.0 {
                    self.update_countdown_ticks += 5;
                }
            }

            self.cooldown = (self.cooldown - 1).max(0);

            // TODO: Add visibility check (canSee) - requires world raycast
            if self.cooldown <= 0
                && mob
                    .get_mob_entity()
                    .is_in_attack_range(target.as_ref())
                    .await
            {
                self.cooldown = self.get_max_cooldown();
                mob.get_mob_entity().living_entity.swing_hand().await;
                mob.get_mob_entity().try_attack(mob, target.as_ref()).await;
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
