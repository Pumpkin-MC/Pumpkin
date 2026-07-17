use std::sync::Arc;

use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob, player::Player};

pub struct TradeWithPlayerGoal {
    speed: f64,
    player: Option<Arc<Player>>,
    update_countdown: i32,
}

impl TradeWithPlayerGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            player: None,
            update_countdown: 0,
        }
    }

    fn trading_player_in_range(mob: &dyn Mob) -> Option<Arc<Player>> {
        let player = mob.get_trading_player()?;
        let entity = &mob.get_mob_entity().living_entity.entity;
        (entity.is_alive()
            && player.get_entity().is_alive()
            && !entity
                .touching_water
                .load(std::sync::atomic::Ordering::Relaxed)
            && entity
                .pos
                .load()
                .squared_distance_to_vec(&player.get_entity().pos.load())
                <= 16.0)
            .then_some(player)
    }

    fn follow_player(&mut self, mob: &dyn Mob) {
        let Some(player) = &self.player else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let player_entity = player.get_entity();
        let player_position = player_entity.pos.load();
        mob_entity.look_control.lock().unwrap().look_at(
            mob,
            player_position.x,
            player_entity.get_eye_y(),
            player_position.z,
        );

        let mob_position = mob_entity.living_entity.entity.pos.load();
        if mob_position.squared_distance_to_vec(&player_position) <= 4.0 {
            mob_entity.navigator.lock().unwrap().stop();
        } else if self.update_countdown <= 0 {
            mob_entity
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal::new(
                    mob_position,
                    Vector3::new(player_position.x, player_position.y, player_position.z),
                    self.speed,
                ));
            self.update_countdown = to_goal_ticks(10);
        }
    }
}

impl Goal for TradeWithPlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.player = Self::trading_player_in_range(mob);
            self.player.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(current) = Self::trading_player_in_range(mob) else {
                return false;
            };
            self.player.as_ref().is_some_and(|player| {
                player.get_entity().entity_uuid == current.get_entity().entity_uuid
            })
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.update_countdown = 0;
            self.follow_player(mob);
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
            self.update_countdown -= 1;
            self.follow_player(mob);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
