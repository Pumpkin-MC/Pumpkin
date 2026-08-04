use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

/// Makes a mob walk back towards its `position_target` (vanilla: `homePosition`) once it strays
/// outside `position_target_range` (vanilla: `homeRadius`).
///
/// Vanilla source: `net/minecraft/world/entity/ai/goal/MoveTowardsRestrictionGoal.java`.
/// Registered on `Blaze` (priority 5), `Guardian` (priority 5) and `WanderingTrader` (priority 4).
///
/// Note: nothing in Pumpkin currently calls `MobEntity::position_target`/`position_target_range`
/// setters (mirroring `Mob.java`'s own `restrictTo`, which most vanilla mobs never call either),
/// so `position_target_range` stays at its `-1` "unrestricted" sentinel and this goal is
/// faithfully dormant, exactly as it is for most vanilla mobs that carry it but are never given
/// a restriction. It's still correct to register: whenever something does start restricting a
/// mob's home area, the goal is immediately live.
///
/// Scope reduction: vanilla picks a path-aware point via
/// `DefaultRandomPos.getPosTowards(mob, 16, 7, homePos, PI/2)`, which walks the pathfinder's
/// random-position search biased towards a target direction within an angle. Pumpkin has no
/// equivalent helper (`WanderAroundGoal::find_wander_target` is an undirected random offset), so
/// this port approximates it: aim roughly at `position_target` with `+/-45` degree horizontal
/// jitter and a random distance, without validating the destination against the navmesh -- the
/// navigator itself will fail out via `should_continue` if it can't path there.
pub struct MoveTowardsRestrictionGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
}

impl MoveTowardsRestrictionGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
        })
    }

    fn find_target(mob: &dyn Mob) -> Vector3<f64> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let home = mob_entity.position_target.load().to_f64();

        let dx = home.x - pos.x;
        let dz = home.z - pos.z;
        let base_angle = dz.atan2(dx);

        let mut rng = mob.get_random();
        // Vanilla's `angleRange` is PI/2 total (i.e. +/-45 degrees from the target direction).
        let jitter = rng.random_range(-std::f64::consts::FRAC_PI_4..=std::f64::consts::FRAC_PI_4);
        let angle = base_angle + jitter;
        let horizontal_dist = rng.random_range(4.0..=16.0);
        let vertical = rng.random_range(-7.0..=7.0);

        Vector3::new(
            pos.x + angle.cos() * horizontal_dist,
            pos.y + vertical,
            pos.z + angle.sin() * horizontal_dist,
        )
    }
}

impl Goal for MoveTowardsRestrictionGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_mob_entity().is_in_position_target_range() {
                return false;
            }

            self.target = Some(Self::find_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !mob.get_mob_entity().navigator.lock().unwrap().is_idle() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
