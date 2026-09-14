use std::ops::BitOr;

use pumpkin_data::entity::EntityStatus;
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::mob::Mob;

pub const OFFER_TICKS: i32 = 400;

pub struct OfferFlowerGoal {
    pub target_entity_id: Option<i32>,
    pub tick: i32,
}

impl Default for OfferFlowerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl OfferFlowerGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            target_entity_id: None,
            tick: 0,
        }
    }
}

impl Goal for OfferFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        let is_night = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_night();
        if is_night {
            return false;
        }

        if mob.get_random().random_range(0..8000) != 0 {
            return false;
        }

        let golem_entity = mob.get_entity();
        let golem_pos = golem_entity.pos.load();
        let bb = golem_entity.bounding_box.load().expand(6.0, 2.0, 6.0);
        let nearby = world.get_entities_at_box(&bb);

        let mut closest: Option<(i32, f64)> = None;

        for candidate in nearby {
            let cand_entity = candidate.get_entity();
            if cand_entity.entity_id == golem_entity.entity_id {
                continue;
            }

            if !cand_entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_CANDIDATE_FOR_IRON_GOLEM_GIFT)
            {
                continue;
            }

            let cand_pos = cand_entity.pos.load();
            let dx = cand_pos.x - golem_pos.x;
            let dy = cand_pos.y - golem_pos.y;
            let dz = cand_pos.z - golem_pos.z;
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq <= 36.0 {
                if let Some((_, closest_dist)) = closest {
                    if dist_sq < closest_dist {
                        closest = Some((cand_entity.entity_id, dist_sq));
                    }
                } else {
                    closest = Some((cand_entity.entity_id, dist_sq));
                }
            }
        }

        if let Some((id, _)) = closest {
            self.target_entity_id = Some(id);
            true
        } else {
            self.target_entity_id = None;
            false
        }
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.tick > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.tick = OFFER_TICKS;
        if let Some(golem) = mob.as_iron_golem() {
            golem.offer_flower(true);
        } else {
            let entity = mob.get_entity();
            let world = entity.world.load();
            world.send_entity_status(entity, EntityStatus::OfferFlower, None);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(golem) = mob.as_iron_golem() {
            golem.offer_flower(false);
        } else {
            let entity = mob.get_entity();
            let world = entity.world.load();
            world.send_entity_status(entity, EntityStatus::StopOfferFlower, None);
        }

        if self.tick == 0
            && let Some(target_id) = self.target_entity_id
        {
            let world = mob.get_entity().world.load();
            if let Some(target) = world.get_entity_by_id(target_id) {
                let target_entity = target.get_entity();
                let bb = mob.get_entity().bounding_box.load().expand(6.0, 2.0, 6.0);
                if target_entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_ACCEPTS_IRON_GOLEM_GIFT)
                    && bb.intersects(&target_entity.bounding_box.load())
                {
                    // Target accepted gift
                }
            }
        }

        self.target_entity_id = None;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(target_id) = self.target_entity_id {
            let world = mob.get_entity().world.load();
            if let Some(target) = world.get_entity_by_id(target_id) {
                let target_entity = target.get_entity();
                let target_pos = target_entity.pos.load();
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .look_at_with_range(
                        target_pos.x,
                        target_entity.get_eye_y(),
                        target_pos.z,
                        30.0,
                        30.0,
                    );
            }
        }
        self.tick -= 1;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE.bitor(Controls::LOOK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offer_flower_goal_lifecycle() {
        let mut goal = OfferFlowerGoal::new();
        assert_eq!(goal.tick, 0);
        assert!(goal.target_entity_id.is_none());

        goal.tick = OFFER_TICKS;
        assert_eq!(goal.tick, 400);
        let controls = goal.controls();
        assert!(controls.get(Controls::MOVE));
        assert!(controls.get(Controls::LOOK));
    }
}
