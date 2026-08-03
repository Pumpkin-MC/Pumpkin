use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_data::data_component_impl::KineticWeaponImpl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// `SpearUseGoal.MIN_REPOSITION_DISTANCE` / `MAX_REPOSITION_DISTANCE`: distance band the mob
/// backs off to mid-charge, before re-closing for the stab.
const MIN_REPOSITION_DISTANCE: f64 = 6.0;
const MAX_REPOSITION_DISTANCE: f64 = 7.0;
/// `SpearUseGoal.MIN_COOLDOWN_DISTANCE` / `MAX_COOLDOWN_DISTANCE`: retreat band used once the
/// stab has landed, keeping the mob clear of normal melee range while it recovers.
const MIN_COOLDOWN_DISTANCE: f64 = 9.0;
const MAX_COOLDOWN_DISTANCE: f64 = 11.0;
/// `SpearUseGoal.MAX_FLEEING_TIME = reducedTickDelay(100)` (`Mth.positiveCeilDiv(100, 2)`).
const MAX_FLEEING_TIME: i32 = 50;
/// `LandRandomPos.getPosAway` vertical search radius argument (`7`) passed by `SpearUseGoal`.
const REPOSITION_VERTICAL_RANGE: i32 = 7;

/// Vanilla derives the windup duration from the held item's `KineticWeapon` data component
/// (`delayTicks + damageConditions.maxDurationTicks`, `reducedTickDelay`'d). Pumpkin's
/// `KineticWeaponImpl` is a marker with no stored per-material fields, so every spear uses a
/// single fixed duration instead: the `wooden_spear` baseline from `Item.Properties.spear`
/// (`delay=0.75*20=15`, `damageTime=4.6*20=92`, total `107`, `reducedTickDelay(107) = 54`).
const ENGAGE_TICKS: i32 = 54;

struct SpearState {
    engage_time: i32,
    fleeing_time: i32,
    away_pos: Option<Vector3<f64>>,
    done: bool,
}

impl SpearState {
    const fn new() -> Self {
        Self {
            engage_time: -1,
            fleeing_time: -1,
            away_pos: None,
            done: false,
        }
    }

    const fn not_engaged_yet(&self) -> bool {
        self.engage_time < 0
    }

    const fn start_engagement(&mut self, ticks: i32) {
        self.engage_time = ticks;
    }

    const fn tick_and_check_engagement(&mut self) -> bool {
        if self.engage_time > 0 {
            self.engage_time -= 1;
            if self.engage_time == 0 {
                return true;
            }
        }
        false
    }

    const fn tick_and_check_fleeing(&mut self) -> bool {
        if self.fleeing_time > 0 {
            self.fleeing_time += 1;
            if self.fleeing_time > MAX_FLEEING_TIME {
                self.done = true;
                return true;
            }
        }
        false
    }
}

/// Port of vanilla's `SpearUseGoal`: approach to spear range, thrust, then reposition/retreat
/// so the mob isn't standing in normal melee range while its spear recovers.
pub struct SpearUseGoal {
    charging_speed: f64,
    repositioning_speed: f64,
    approach_distance_sq: f64,
    target_in_range_radius_sq: f64,
    state: Option<SpearState>,
}

impl SpearUseGoal {
    #[must_use]
    pub fn new(
        charging_speed: f64,
        repositioning_speed: f64,
        approach_distance: f64,
        target_in_range_radius: f64,
    ) -> Box<Self> {
        Box::new(Self {
            charging_speed,
            repositioning_speed,
            approach_distance_sq: approach_distance * approach_distance,
            target_in_range_radius_sq: target_in_range_radius * target_in_range_radius,
            state: None,
        })
    }

    async fn able_to_attack(mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().target.lock().await;
        let Some(target) = target.as_ref() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }

        let living_entity = &mob.get_mob_entity().living_entity;
        let held_item = living_entity.held_item(&living_entity.entity).await;
        let held_item = held_item.lock().await;
        held_item
            .get_data_component::<KineticWeaponImpl>()
            .is_some()
    }

    fn move_to(mob: &dyn Mob, destination: Vector3<f64>, speed: f64) {
        let mob_pos = mob.get_entity().pos.load();
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap()
            .set_progress(NavigatorGoal::new(mob_pos, destination, speed));
    }

    fn navigator_idle(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().navigator.lock().unwrap().is_idle()
    }

    /// Simplified port of `LandRandomPos.getPosAway`: samples a point within a +/-90 degree
    /// cone of the direction away from `avoid_pos`, at a distance in `[min_dist, max_dist]`,
    /// and rejects candidates that aren't standable ground. Vanilla additionally validates
    /// candidates against the mob's node evaluator (path stability, malus, restrictions);
    /// this uses the same solid-ground/passable check as `AvoidEntityGoal::find_flee_position`
    /// instead, since porting the full node-evaluator-backed random-position search is out of
    /// scope for this goal.
    fn find_position_away(
        mob: &dyn Mob,
        avoid_pos: &Vector3<f64>,
        min_dist: f64,
        max_dist: f64,
    ) -> Option<Vector3<f64>> {
        let entity = mob.get_entity();
        let mob_pos = entity.pos.load();
        let world = entity.world.load();

        let candidates = {
            let mut rng = mob.get_random();
            let dir_x = mob_pos.x - avoid_pos.x;
            let dir_z = mob_pos.z - avoid_pos.z;
            let (dir_x, dir_z) = if dir_x == 0.0 && dir_z == 0.0 {
                (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
            } else {
                (dir_x, dir_z)
            };
            let base_angle = dir_z.atan2(dir_x) - std::f64::consts::FRAC_PI_2;

            let mut candidates = Vec::with_capacity(10);
            for _ in 0..10 {
                let angle = base_angle
                    + (2.0 * rng.random_range(0.0..1.0) - 1.0) * std::f64::consts::FRAC_PI_2;
                let dist = rng.random_range(min_dist..=max_dist.max(min_dist));
                let dx = -dist * angle.sin();
                let dz = dist * angle.cos();
                let dy = rng.random_range(-REPOSITION_VERTICAL_RANGE..=REPOSITION_VERTICAL_RANGE);
                candidates.push((dx, dy, dz));
            }
            candidates
        };

        for (dx, dy, dz) in candidates {
            let candidate = BlockPos::new(
                (mob_pos.x + dx) as i32,
                (mob_pos.y + f64::from(dy)) as i32,
                (mob_pos.z + dz) as i32,
            );

            let block_at = world.get_block_state(&candidate);
            let block_below = world.get_block_state(&BlockPos::new(
                candidate.0.x,
                candidate.0.y - 1,
                candidate.0.z,
            ));

            if block_at.is_solid() || !block_below.is_solid() {
                continue;
            }

            return Some(Vector3::new(
                f64::from(candidate.0.x) + 0.5,
                f64::from(candidate.0.y),
                f64::from(candidate.0.z) + 0.5,
            ));
        }

        None
    }
}

impl Goal for SpearUseGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::able_to_attack(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            match &self.state {
                Some(state) if !state.done => Self::able_to_attack(mob).await,
                _ => false,
            }
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_attacking(true);
            self.state = Some(SpearState::new());
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            mob.get_mob_entity().set_attacking(false);
            self.state = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            let Some(state) = self.state.as_mut() else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let target_dist_sq = mob_pos.squared_distance_to_vec(&target_pos);
            let mount_distance = if mob.get_entity().has_vehicle().await {
                2.0
            } else {
                0.0
            };

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            if state.not_engaged_yet() {
                if target_dist_sq > self.approach_distance_sq {
                    Self::move_to(mob, target_pos, self.repositioning_speed);
                    return;
                }
                state.start_engagement(ENGAGE_TICKS);
            }

            if state.tick_and_check_engagement() {
                if mob
                    .get_mob_entity()
                    .is_in_attack_range(target.as_ref())
                    .await
                {
                    mob.get_mob_entity().living_entity.swing_hand().await;
                    mob.try_attack(target.as_ref()).await;
                }

                let distance = target_dist_sq.sqrt();
                state.away_pos = Self::find_position_away(
                    mob,
                    &target_pos,
                    (MIN_COOLDOWN_DISTANCE + mount_distance - distance).max(0.0),
                    (MAX_COOLDOWN_DISTANCE + mount_distance - distance).max(1.0),
                );
                state.fleeing_time = 1;
            }

            if !state.tick_and_check_fleeing() {
                if let Some(away_pos) = state.away_pos {
                    Self::move_to(mob, away_pos, self.repositioning_speed);
                    if Self::navigator_idle(mob) {
                        if state.fleeing_time > 0 {
                            state.done = true;
                            return;
                        }
                        state.away_pos = None;
                    }
                } else {
                    Self::move_to(mob, target_pos, self.charging_speed);
                    if target_dist_sq < self.target_in_range_radius_sq || Self::navigator_idle(mob)
                    {
                        let distance = target_dist_sq.sqrt();
                        state.away_pos = Self::find_position_away(
                            mob,
                            &target_pos,
                            (MIN_REPOSITION_DISTANCE + mount_distance - distance).max(0.0),
                            (MAX_REPOSITION_DISTANCE + mount_distance - distance).max(1.0),
                        );
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::{MAX_FLEEING_TIME, SpearState};

    #[test]
    fn starts_unengaged_and_not_fleeing() {
        let state = SpearState::new();
        assert!(state.not_engaged_yet());
        assert!(!state.done);
    }

    #[test]
    fn engagement_fires_exactly_once_when_the_countdown_reaches_zero() {
        let mut state = SpearState::new();
        state.start_engagement(3);
        assert!(!state.not_engaged_yet());
        assert!(!state.tick_and_check_engagement());
        assert!(!state.tick_and_check_engagement());
        assert!(state.tick_and_check_engagement());
        // Countdown is spent; further ticks must not fire again.
        assert!(!state.tick_and_check_engagement());
    }

    #[test]
    fn engagement_of_zero_ticks_never_fires() {
        // Vanilla: `tickAndCheckEngagement` only returns true when it decrements *down to*
        // zero; starting already at zero never enters the `engageTime > 0` branch.
        let mut state = SpearState::new();
        state.start_engagement(0);
        assert!(!state.tick_and_check_engagement());
    }

    #[test]
    fn fleeing_is_inert_until_started() {
        let mut state = SpearState::new();
        assert!(!state.tick_and_check_fleeing());
        assert!(!state.done);
    }

    #[test]
    fn fleeing_times_out_after_max_fleeing_time() {
        let mut state = SpearState::new();
        state.fleeing_time = 1;
        for _ in 0..MAX_FLEEING_TIME - 1 {
            assert!(!state.tick_and_check_fleeing());
        }
        assert!(state.tick_and_check_fleeing());
        assert!(state.done);
    }
}
