use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::{EndermanEntity, PLAYER_EYE_HEIGHT};
use crate::entity::player::Player;

pub struct ChasePlayerGoal {
    enderman: Arc<EndermanEntity>,
    target: Option<Arc<Player>>,
}

impl ChasePlayerGoal {
    pub const fn new(enderman: Arc<EndermanEntity>) -> Self {
        Self {
            enderman,
            target: None,
        }
    }
}

impl Goal for ChasePlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let target = mob_entity.target.lock().await.clone();

            let Some(target) = target else {
                self.target = None;
                return false;
            };

            let Some(player) = target.get_player() else {
                self.target = None;
                return false;
            };

            let entity = &mob_entity.living_entity.entity;
            let mob_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(&target_pos) > 256.0 {
                self.target = None;
                return false;
            }

            if !self.enderman.is_player_staring(player).await {
                self.target = None;
                return false;
            }

            let world = entity.world.load();
            let closest = world.get_closest_player(mob_pos, 256.0);
            if let Some(p) = closest
                && p.get_entity().entity_id == target.get_entity().entity_id
            {
                self.target = Some(p);
                return true;
            }

            self.target = None;
            false
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(player) = &self.target else {
                return false;
            };

            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let mob_pos = entity.pos.load();
            let target_pos = player.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(&target_pos) > 256.0 {
                return false;
            }

            self.enderman.is_player_staring(player).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = &self.target {
                let player_pos = player.get_entity().pos.load();
                let eye_y = player_pos.y + PLAYER_EYE_HEIGHT;
                let mut look_control = mob.get_mob_entity().look_control.lock().await;
                look_control.look_at(mob, player_pos.x, eye_y, player_pos.z);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
