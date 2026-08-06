use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use pumpkin_util::math::vector3::Vector3;

use super::fox_util::fox_path_is_clear;
use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::passive::fox::FoxEntity;

const POUNCE_RANGE_SQ: f64 = 4.0;

/// `Fox.FoxPounceGoal` (`JumpGoal`): once fully crouched, leaps straight at the target and
/// bites on contact.
///
/// Two vanilla details are dropped as documented simplifications:
/// - `target.getMotionDirection() != target.getDirection()` (the target must be moving
///   forward, not just turning in place) has no equivalent on this codebase's `EntityBase`,
///   which doesn't track a separate "motion direction" from facing.
/// - The mid-air `xRot` pounce-arc animation is purely cosmetic and not tracked server-side;
///   vanilla derives the snow-faceplant trigger from it (`xRot > 0.0` while landing), so here
///   the faceplant trigger is simplified to "landed on a snow block with residual vertical
///   velocity" instead.
pub struct FoxPounceGoal;

impl FoxPounceGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Goal for FoxPounceGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return false;
            };
            if !fox.is_fully_crouched() {
                return false;
            }
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }

            let world = mob.get_entity().world.load();
            let my_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let clear = fox_path_is_clear(&world, my_pos, target_pos);
            if !clear {
                fox.set_is_crouching(false);
                fox.set_is_interested(false);
            }
            clear
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return false;
            };
            if fox.is_faceplanted() {
                return false;
            }
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }

            let entity = mob.get_entity();
            let velocity = entity.velocity.load();
            !(velocity.y * velocity.y < 0.05 && entity.on_ground.load(Relaxed))
        })
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return;
            };
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(true, SeqCst);
            fox.set_is_pouncing(true);
            fox.set_is_interested(false);

            let target = mob.get_mob_entity().target.lock().await.clone();
            if let Some(target) = target {
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_entity(mob, &target);

                let entity = mob.get_entity();
                let my_pos = entity.pos.load();
                let target_pos = target.get_entity().pos.load();
                let dir = Vector3::new(
                    target_pos.x - my_pos.x,
                    target_pos.y - my_pos.y,
                    target_pos.z - my_pos.z,
                )
                .normalize();
                let velocity = entity.velocity.load();
                entity.set_velocity(Vector3::new(
                    velocity.x + dir.x * 0.8,
                    velocity.y + 0.9,
                    velocity.z + dir.z * 0.8,
                ));
            }

            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                fox.set_is_crouching(false);
                fox.set_is_interested(false);
                fox.set_is_pouncing(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return;
            };
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity(mob, &target);

            let entity = mob.get_entity();
            let my_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();

            if my_pos.squared_distance_to_vec(&target_pos) <= POUNCE_RANGE_SQ {
                mob.try_attack(target.as_ref()).await;
                return;
            }

            if entity.on_ground.load(Relaxed) {
                let velocity = entity.velocity.load();
                if velocity.y != 0.0 {
                    let world = entity.world.load();
                    let pos = entity.block_pos.load();
                    if world.get_block_and_state(&pos).0 == &pumpkin_data::Block::SNOW {
                        mob.get_mob_entity().target.lock().await.take();
                        fox.set_faceplanted(true);
                    }
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}
