use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VillagerActivity {
    Idle,
    Work,
    Meet,
    Rest,
}

/// `data/minecraft/timeline/villager_schedule.json`, track
/// `minecraft:gameplay/villager_activity` (period 24000 ticks):
/// idle@10, work@2000, meet@9000, idle@11000, rest@12000.
///
/// The baby schedule (`minecraft:gameplay/baby_villager_activity`, which
/// swaps work/meet for play@3000/10000) is not modeled: Pumpkin villagers
/// don't hold job sites as babies, so there is nothing for a baby-specific
/// schedule to gate here.
#[must_use]
pub const fn villager_activity_for_time(time_of_day: i64) -> VillagerActivity {
    let t = time_of_day.rem_euclid(24000);
    if t < 2000 {
        VillagerActivity::Idle
    } else if t < 9000 {
        VillagerActivity::Work
    } else if t < 11000 {
        VillagerActivity::Meet
    } else if t < 12000 {
        VillagerActivity::Idle
    } else {
        VillagerActivity::Rest
    }
}

const ARRIVAL_DIST_SQR: f64 = 4.0;
const REPATH_INTERVAL: i32 = 40;

/// Simplified stand-in for vanilla's `UpdateActivityFromSchedule` +
/// `getWorkPackage`/`getRestPackage` (`VillagerGoalPackages.java`).
///
/// Vanilla drives this through the Brain/Activity/Memory framework (`WorkAtPoi`,
/// `StrollAroundPoi`, `SetWalkTargetFromBlockMemory`, `SleepInBed`, ...), which Pumpkin has
/// no equivalent of. This goal instead just walks the villager to its already-claimed job
/// site during WORK hours and to its already-claimed bed during REST hours, reusing the
/// job-site/home tracking `VillagerEntity` already does in `mob_tick`. During MEET/IDLE
/// hours it does nothing, which is a control-conflict no-op: with `Controls::MOVE` and a
/// lower (more urgent) priority number than `WanderAroundGoal`, this goal preempts
/// wandering whenever it is active and yields `MOVE` back to `WanderAroundGoal` otherwise --
/// so the villager wanders during MEET/IDLE and walks to work/bed on schedule the rest of
/// the day, without needing to touch `WanderAroundGoal` itself (shared by dozens of
/// unrelated mobs).
pub struct VillagerScheduleGoal {
    speed: f64,
    target: Option<Vector3<f64>>,
    ticks_since_path: i32,
}

impl VillagerScheduleGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
            ticks_since_path: 0,
        }
    }

    async fn scheduled_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let time = world.get_time_of_day().await;
        let pos = match villager_activity_for_time(time) {
            VillagerActivity::Work => mob.get_job_site(),
            VillagerActivity::Rest => mob.get_home(),
            VillagerActivity::Idle | VillagerActivity::Meet => None,
        }?;
        Some(Vector3::new(
            pos.0.x as f64 + 0.5,
            pos.0.y as f64,
            pos.0.z as f64 + 0.5,
        ))
    }

    fn has_arrived(target: Vector3<f64>, mob: &dyn Mob) -> bool {
        target.squared_distance_to_vec(&mob.get_entity().pos.load()) <= ARRIVAL_DIST_SQR
    }
}

impl Goal for VillagerScheduleGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = Self::scheduled_target(mob).await else {
                return false;
            };
            if Self::has_arrived(target, mob) {
                return false;
            }
            self.target = Some(target);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = self.target else {
                return false;
            };
            if Self::has_arrived(target, mob) {
                return false;
            }
            Self::scheduled_target(mob).await == Some(target)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_since_path = 0;
            if let Some(target) = self.target {
                let pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_since_path += 1;
            if self.ticks_since_path % REPATH_INTERVAL != 0 {
                return;
            }
            let Some(target) = self.target else {
                return;
            };
            let is_idle = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
            if is_idle {
                let pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn activity_windows_match_villager_schedule_json() {
        assert_eq!(villager_activity_for_time(0), VillagerActivity::Idle);
        assert_eq!(villager_activity_for_time(10), VillagerActivity::Idle);
        assert_eq!(villager_activity_for_time(1999), VillagerActivity::Idle);
        assert_eq!(villager_activity_for_time(2000), VillagerActivity::Work);
        assert_eq!(villager_activity_for_time(8999), VillagerActivity::Work);
        assert_eq!(villager_activity_for_time(9000), VillagerActivity::Meet);
        assert_eq!(villager_activity_for_time(10999), VillagerActivity::Meet);
        assert_eq!(villager_activity_for_time(11000), VillagerActivity::Idle);
        assert_eq!(villager_activity_for_time(11999), VillagerActivity::Idle);
        assert_eq!(villager_activity_for_time(12000), VillagerActivity::Rest);
        assert_eq!(villager_activity_for_time(23999), VillagerActivity::Rest);
    }

    #[test]
    fn activity_wraps_across_multiple_days() {
        assert_eq!(
            villager_activity_for_time(24000 + 2000),
            VillagerActivity::Work
        );
        assert_eq!(
            villager_activity_for_time(3 * 24000 + 12000),
            VillagerActivity::Rest
        );
    }

    #[test]
    fn activity_handles_negative_time_of_day() {
        assert_eq!(villager_activity_for_time(-1), VillagerActivity::Rest);
        assert_eq!(villager_activity_for_time(-24000), VillagerActivity::Idle);
    }
}
