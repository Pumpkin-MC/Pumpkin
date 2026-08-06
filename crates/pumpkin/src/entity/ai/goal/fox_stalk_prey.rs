use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;

use super::fox_util::fox_path_is_clear;
use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::fox::FoxEntity;

const STALK_RANGE_SQ: f64 = 36.0;
const STALK_SPEED: f64 = 1.5;

async fn stalkable_target(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
    let target = mob.get_mob_entity().target.lock().await.clone()?;
    if !target.get_entity().is_alive() {
        return None;
    }
    let target_type = target.get_entity().entity_type;
    (target_type == &EntityType::CHICKEN || target_type == &EntityType::RABBIT).then_some(target)
}

/// `Fox.StalkPreyGoal`: crouches and stares at a targeted chicken/rabbit once within range.
///
/// Closes the distance otherwise. Requires the fox's `target` to already be set to a chicken or
/// rabbit; wiring the target-selector that finds that prey in the first place (this codebase's
/// `ActiveTargetGoal`) happens in `FoxEntity::new`, alongside the sleep/crouch/pounce goals.
pub struct StalkPreyGoal;

impl StalkPreyGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Goal for StalkPreyGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return false;
            };
            if fox.is_sleeping() || fox.is_crouching() || fox.is_interested() {
                return false;
            }
            if mob.get_mob_entity().living_entity.jumping.load(Relaxed) {
                return false;
            }
            let Some(target) = stalkable_target(mob).await else {
                return false;
            };
            let my_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            my_pos.squared_distance_to_vec(&target_pos) > STALK_RANGE_SQ
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                fox.set_sitting(false);
                fox.set_faceplanted(false);
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return;
            };
            let target = mob.get_mob_entity().target.lock().await.clone();
            let world = mob.get_entity().world.load();
            let my_pos = mob.get_entity().pos.load();

            let clear = target.as_ref().is_some_and(|target| {
                target.get_entity().is_alive()
                    && fox_path_is_clear(&world, my_pos, target.get_entity().pos.load())
            });

            if clear {
                let target = target.unwrap();
                fox.set_is_interested(true);
                fox.set_is_crouching(true);
                mob.get_mob_entity().navigator.lock().unwrap().stop();
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_entity(mob, &target);
            } else {
                fox.set_is_interested(false);
                fox.set_is_crouching(false);
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

            let my_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();

            if my_pos.squared_distance_to_vec(&target_pos) <= STALK_RANGE_SQ {
                fox.set_is_interested(true);
                fox.set_is_crouching(true);
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            } else {
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal::new(my_pos, target_pos, STALK_SPEED));
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
