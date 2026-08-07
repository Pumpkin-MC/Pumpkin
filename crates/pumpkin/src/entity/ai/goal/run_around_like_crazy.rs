use std::sync::Weak;

use pumpkin_data::entity::EntityStatus;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, passive::equine::AbstractHorse};

/// `RunAroundLikeCrazyGoal`'s `DefaultRandomPos.getPos(horse, 5, 4)` horizontal/vertical search
/// radii. Simplified the same way `EscapeDangerGoal`/`SilverfishMergeWithStoneGoal` simplify
/// `DefaultRandomPos` elsewhere in this codebase: a direct random offset rather than a
/// pathfinder-validated candidate search.
const HORIZONTAL_RANGE: f64 = 5.0;
const VERTICAL_RANGE: f64 = 4.0;

/// `RunAroundLikeCrazyGoal.java`'s `nextInt(adjustedTickDelay(50)) == 0` buck-check roll.
const BUCK_CHECK_INTERVAL: i32 = 50;

/// `AbstractHorse.modifyTemper` amount applied on a failed taming roll.
const TEMPER_GAIN_ON_FAILURE: i32 = 5;

/// Vanilla `RunAroundLikeCrazyGoal`: while an untamed horse-family mob is being ridden, it
/// bolts to a random nearby point.
///
/// On a periodic roll it either tames (chance scales with
/// accumulated temper) or throws the rider and rears up angrily.
///
/// Generic over the concrete horse-family species and `?Sized` so `Weak<dyn LlamaMob>` also
/// fits (`register_llama_goals` only has a type-erased `Arc<dyn Mob>` to build a handle from) --
/// same pattern `AmbientStandGoal` uses for the concrete species.
pub struct RunAroundLikeCrazyGoal<T: ?Sized> {
    horse: Weak<T>,
    speed: f64,
    target: Option<Vector3<f64>>,
}

impl<T: AbstractHorse + Mob + ?Sized> RunAroundLikeCrazyGoal<T> {
    #[must_use]
    pub fn new(horse: Weak<T>, speed: f64) -> Box<Self> {
        Box::new(Self {
            horse,
            speed,
            target: None,
        })
    }

    /// `DefaultRandomPos.getPos(horse, 5, 4)`, simplified per the module doc comment.
    fn find_target(mob: &dyn Mob) -> Vector3<f64> {
        let pos = mob.get_entity().pos.load();
        let mut rng = mob.get_random();
        let dx = rng.random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE);
        let dy = rng.random_range(-VERTICAL_RANGE..=VERTICAL_RANGE);
        let dz = rng.random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE);
        Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz)
    }
}

impl<T: AbstractHorse + Mob + ?Sized + Send + Sync + 'static> Goal for RunAroundLikeCrazyGoal<T> {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(horse) = self.horse.upgrade() else {
                return false;
            };

            if horse.is_mob_controlled().await || horse.is_tamed() {
                return false;
            }
            if !mob.get_entity().has_passengers().await {
                return false;
            }

            self.target = Some(Self::find_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(horse) = self.horse.upgrade() else {
                return false;
            };
            if horse.is_tamed() {
                return false;
            }
            if mob.get_mob_entity().navigator.lock().unwrap().is_idle() {
                return false;
            }
            mob.get_entity().has_passengers().await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(horse) = self.horse.upgrade() else {
                return;
            };
            if horse.is_tamed() {
                return;
            }
            if mob
                .get_random()
                .random_range(0..to_goal_ticks(BUCK_CHECK_INTERVAL))
                != 0
            {
                return;
            }

            let entity = mob.get_entity();
            let Some(passenger) = entity.passengers.lock().await.first().cloned() else {
                return;
            };

            if let Some(player) = passenger.get_player() {
                let temper = horse.get_temper();
                let max_temper = horse.max_temper();
                if max_temper > 0 && mob.get_random().random_range(0..max_temper) < temper {
                    horse.set_tamed(player.gameprofile.id);
                    return;
                }
                horse.modify_temper(TEMPER_GAIN_ON_FAILURE);
            }

            let passengers = entity.passengers.lock().await.clone();
            for passenger in passengers {
                entity
                    .remove_passenger(passenger.get_entity().entity_id)
                    .await;
            }

            horse.make_mad();
            let world = entity.world.load();
            world.send_entity_status(entity, EntityStatus::TamingFailed);
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
