use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::{EndermanEntity, PLAYER_EYE_HEIGHT};

pub struct ChasePlayerGoal {
    enderman: Arc<EndermanEntity>,
    target: Option<Arc<dyn EntityBase>>,
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
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let target = mob_entity.get_target();

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

        if !self.enderman.is_player_staring(player) {
            self.target = None;
            return false;
        }

        self.target = Some(target);
        true
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(target) = &self.target {
            let player_pos = target.get_entity().pos.load();
            let eye_y = player_pos.y + PLAYER_EYE_HEIGHT;
            let mut look_control = mob
                .get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look_control.look_at(mob, player_pos.x, eye_y, player_pos.z);
        }
    }

    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
