use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::player::Player;

const START_RANGE: f64 = 10.0;
const CONTINUE_RANGE_SQ: f64 = 16.0 * 16.0;
const STOP_RANGE_SQ: f64 = 2.5 * 2.5;

/// Vanilla `Dolphin.DolphinSwimWithPlayerGoal`.
pub struct DolphinSwimWithPlayerGoal {
    speed: f64,
    player: Option<Arc<Player>>,
}

impl DolphinSwimWithPlayerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            player: None,
        })
    }

    fn is_swimming(player: &Player) -> bool {
        let entity = player.get_entity();
        // Player swimming state is not fully synchronized yet; water contact is the
        // existing server-side approximation used by Player movement.
        entity.swimming.load(std::sync::atomic::Ordering::Relaxed)
            || entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn find_player(mob: &dyn Mob) -> Option<Arc<Player>> {
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load();

        world
            .get_nearby_players(pos, START_RANGE)
            .into_iter()
            .filter(|player| !player.is_spectator() && Self::is_swimming(player))
            .min_by(|left, right| {
                left.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .total_cmp(&right.get_entity().pos.load().squared_distance_to_vec(&pos))
            })
    }

    async fn grant_grace(player: &Player) {
        player
            .add_effect(Effect {
                effect_type: &StatusEffect::DOLPHINS_GRACE,
                duration: 100,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;
    }

    async fn is_target(mob: &dyn Mob, player: &Player) -> bool {
        mob.get_mob_entity()
            .target
            .lock()
            .await
            .as_ref()
            .map(|target| target.get_entity().entity_id)
            .is_some_and(|target_id| target_id == player.get_entity().entity_id)
    }

    fn distance_sq(mob: &dyn Mob, player: &Player) -> f64 {
        mob.get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&player.get_entity().pos.load())
    }
}

impl Goal for DolphinSwimWithPlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(player) = Self::find_player(mob) else {
                return false;
            };
            if Self::is_target(mob, &player).await {
                return false;
            }

            self.player = Some(player);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.player.as_ref().is_some_and(|player| {
                !player.is_spectator()
                    && player.get_entity().is_alive()
                    && Self::is_swimming(player)
                    && Self::distance_sq(mob, player) < CONTINUE_RANGE_SQ
            })
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = &self.player {
                Self::grant_grace(player).await;
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.player = None;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(player) = &self.player else {
                return;
            };

            let player_pos: Vector3<f64> = player.get_entity().pos.load();
            let distance_sq = Self::distance_sq(mob, player);
            let mob_entity = mob.get_mob_entity();
            mob_entity.look_control.lock().unwrap().look_at(
                mob,
                player_pos.x,
                player.get_entity().get_eye_y(),
                player_pos.z,
            );

            let mut navigator = mob_entity.navigator.lock().unwrap();
            if distance_sq < STOP_RANGE_SQ {
                navigator.stop();
            } else {
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
            }
            drop(navigator);

            if Self::is_swimming(player) && mob.get_random().random_range(0..6) == 0 {
                Self::grant_grace(player).await;
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
