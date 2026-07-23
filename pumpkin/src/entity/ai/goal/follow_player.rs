//! Casual follow-nearest-player (Allay stand-in for item-collect companion AI).

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

const START_DIST_SQ: f64 = 4.0 * 4.0;
const STOP_DIST_SQ: f64 = 2.0 * 2.0;
const MAX_DIST: f64 = 16.0;

pub struct FollowPlayerGoal {
    speed: f64,
}

impl FollowPlayerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed: speed.max(0.3),
        })
    }
}

impl Goal for FollowPlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Don't steal combat focus.
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            let pos = mob.get_entity().pos.load();
            let world = mob.get_entity().world.load();
            let Some(player) = world.get_closest_player(pos, MAX_DIST) else {
                return false;
            };
            if player.is_spectator() {
                return false;
            }
            let d = pos.squared_distance_to_vec(&player.position());
            d > START_DIST_SQ
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            let pos = mob.get_entity().pos.load();
            let world = mob.get_entity().world.load();
            let Some(player) = world.get_closest_player(pos, MAX_DIST) else {
                return false;
            };
            if player.is_spectator() {
                return false;
            }
            pos.squared_distance_to_vec(&player.position()) > STOP_DIST_SQ
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let pos = mob.get_entity().pos.load();
            let world = mob.get_entity().world.load();
            let Some(player) = world.get_closest_player(pos, MAX_DIST) else {
                return;
            };
            let dest = player.position();
            {
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at(mob, dest.x, dest.y + 1.0, dest.z);
            }
            let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
            nav.set_progress(NavigatorGoal::new(pos, dest, self.speed));
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
