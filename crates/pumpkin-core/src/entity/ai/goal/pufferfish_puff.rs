use std::sync::Weak;

use super::Goal;

use crate::entity::mob::Mob;
use crate::entity::passive::pufferfish::PufferfishEntity;

pub struct PufferfishPuffGoal {
    puffer: Weak<PufferfishEntity>,
}

impl PufferfishPuffGoal {
    #[must_use]
    pub const fn new(puffer: Weak<PufferfishEntity>) -> Self {
        Self { puffer }
    }
}

impl Goal for PufferfishPuffGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(puffer) = self.puffer.upgrade() else {
            return false;
        };
        let entity = mob.get_entity();
        let world = entity.world.load();
        let bounding_box = entity.bounding_box.load().expand_all(2.0);
        world
            .get_entities_at_box(&bounding_box)
            .iter()
            .any(|target| puffer.is_scary(mob, target.as_ref()))
            || world
                .get_players_at_box(&bounding_box)
                .iter()
                .any(|player| puffer.is_scary(mob, player.as_ref()))
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(puffer) = self.puffer.upgrade() {
            puffer.start_inflating();
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(puffer) = self.puffer.upgrade() {
            puffer.stop_inflating();
        }
    }
}
