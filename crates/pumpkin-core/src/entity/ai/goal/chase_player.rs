use std::sync::{Arc, Weak};

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;

pub struct ChasePlayerGoal {
    enderman: Weak<EndermanEntity>,
    target: Option<Weak<dyn EntityBase>>,
}

impl ChasePlayerGoal {
    #[must_use]
    pub const fn new(enderman: Weak<EndermanEntity>) -> Self {
        Self {
            enderman,
            target: None,
        }
    }
}

impl Goal for ChasePlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(enderman) = self.enderman.upgrade() else {
            self.target = None;
            return false;
        };
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

        if !enderman.is_player_staring(player) {
            self.target = None;
            return false;
        }

        self.target = Some(Arc::downgrade(&target));
        true
    }

    fn start(&mut self, mob: &dyn Mob) {
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target.as_ref().and_then(Weak::upgrade) {
            let player_pos = target.get_entity().pos.load();
            let eye_y = target.get_entity().get_eye_y();
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
