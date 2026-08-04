use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use pumpkin_data::entity::{EntityType, MobCategory};
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::passive::fox::FoxEntity;
use crate::world::World;

const WAIT_TIME_BEFORE_SLEEP: i32 = 140;
const BRIGHT_OUTSIDE_THRESHOLD: u8 = 4;
const ALERT_RADIUS: f64 = 12.0;

fn is_bright_outside(world: &World) -> bool {
    world.dimension.has_skylight && world.sky_darken.load(Relaxed) < BRIGHT_OUTSIDE_THRESHOLD
}

/// `Fox.FoxAlertableEntitiesSelector`/`FoxBehaviorGoal.alertable`: whether anything nearby
/// makes it unsafe to sleep. Vanilla's selector also covers untamed tameable animals and
/// awake, non-discrete players within a wider box; both are dropped here as documented
/// simplifications (no generic "is tameable and not tame" check exists on `EntityBase`, and
/// player-alertness gating is lower value than the monster/prey gate that's kept).
fn alertable(mob: &dyn Mob) -> bool {
    let entity = mob.get_entity();
    let world = entity.world.load();
    let pos = entity.pos.load();
    let self_uuid = entity.entity_uuid;

    world
        .get_nearby_entities(pos, ALERT_RADIUS)
        .into_iter()
        .any(|(uuid, other)| {
            if uuid == self_uuid {
                return false;
            }
            let other_type = other.get_entity().entity_type;
            if other_type == &EntityType::FOX {
                return false;
            }
            other_type.category == &MobCategory::MONSTER
                || other_type == &EntityType::CHICKEN
                || other_type == &EntityType::RABBIT
        })
}

/// `Fox.SleepGoal`: after a randomized wait, sleeps during the day in a sheltered spot with
/// nothing nearby to disturb it.
///
/// Vanilla's `canUse` additionally requires `xxa == 0 && yya == 0 && zza == 0` (no AI
/// movement input currently applied); this codebase has no equivalent per-tick movement-input
/// fields on `Mob`, so it's approximated with "navigator is idle", which is the observable
/// condition that field was gating in practice. `hasShelter` similarly drops vanilla's
/// `getWalkTargetValue(pos) >= 0.0F` half (no such primitive here, same gap `FleeSunGoal`
/// already documents) and keeps only the `!canSeeSky` half.
pub struct FoxSleepGoal {
    countdown: i32,
}

impl FoxSleepGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            countdown: rand::rng().random_range(0..WAIT_TIME_BEFORE_SLEEP),
        })
    }

    fn can_sleep(&mut self, mob: &dyn Mob) -> bool {
        if self.countdown > 0 {
            self.countdown -= 1;
            return false;
        }

        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();

        is_bright_outside(&world)
            && !world.can_see_sky(&pos)
            && !alertable(mob)
            && !entity.is_in_powder_snow.load(Relaxed)
    }
}

impl Goal for FoxSleepGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return false;
            };
            let navigator_idle = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
            if !navigator_idle && !fox.is_sleeping() {
                return false;
            }
            self.can_sleep(mob) || fox.is_sleeping()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_sleep(mob) })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                fox.set_sitting(false);
                fox.set_is_crouching(false);
                fox.set_is_interested(false);
                fox.set_sleeping(true);
            }
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(false, SeqCst);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.countdown = mob.get_random().random_range(0..WAIT_TIME_BEFORE_SLEEP);
            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                fox.clear_states();
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}
