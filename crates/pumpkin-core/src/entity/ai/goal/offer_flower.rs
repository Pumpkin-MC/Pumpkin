use std::ops::BitOr;

use pumpkin_data::entity::EntityStatus;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::boundingbox::BoundingBox;
use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::passive::copper_golem::EQUIPMENT_SLOT_ANTENNA;

pub const OFFER_TICKS: i32 = 400;
const OFFER_TARGET_RANGE: f64 = 6.0;
const OFFER_CHANCE: i32 = 8000;
const BOUNDING_BOX_INFLATE_XZ: f64 = 6.0;
const BOUNDING_BOX_INFLATE_Y: f64 = 2.0;

pub struct OfferFlowerGoal {
    pub target_entity_id: Option<i32>,
    pub tick: i32,
    offer_targeting: TargetPredicate,
}

impl Default for OfferFlowerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl OfferFlowerGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            target_entity_id: None,
            tick: 0,
            offer_targeting: TargetPredicate::create_non_attackable()
                .set_base_max_distance(OFFER_TARGET_RANGE),
        }
    }

    fn golem_bounding_box(mob: &dyn Mob) -> BoundingBox {
        mob.get_entity().bounding_box.load().expand(
            BOUNDING_BOX_INFLATE_XZ,
            BOUNDING_BOX_INFLATE_Y,
            BOUNDING_BOX_INFLATE_XZ,
        )
    }
}

impl Goal for OfferFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        if !world.is_bright_outside() || mob.get_random().random_range(0..OFFER_CHANCE) != 0 {
            return false;
        }

        let position = entity.pos.load();
        let nearest = world
            .get_entities_at_box(&Self::golem_bounding_box(mob))
            .into_iter()
            .filter(|candidate| {
                candidate
                    .get_entity()
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_CANDIDATE_FOR_IRON_GOLEM_GIFT)
                    && self
                        .offer_targeting
                        .test(&world, Some(mob), candidate.as_ref())
            })
            .min_by(|a, b| {
                let distance = |entity: &dyn EntityBase| {
                    entity
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&position)
                };
                distance(a.as_ref()).total_cmp(&distance(b.as_ref()))
            });
        self.target_entity_id = nearest.map(|nearest| nearest.get_entity().entity_id);
        self.target_entity_id.is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.tick > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.tick = self.get_tick_count(OFFER_TICKS);
        if let Some(golem) = mob.as_iron_golem() {
            golem.offer_flower(true);
        } else {
            let entity = mob.get_entity();
            let world = entity.world.load();
            world.send_entity_status(
                entity,
                EntityStatus::OfferFlower,
                ActorEventID::StartOfferFlower,
            );
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(golem) = mob.as_iron_golem() {
            golem.offer_flower(false);
        } else {
            let entity = mob.get_entity();
            let world = entity.world.load();
            world.send_entity_status(
                entity,
                EntityStatus::StopOfferFlower,
                ActorEventID::StopOfferFlower,
            );
        }

        if self.tick == 0
            && let Some(target_id) = self.target_entity_id
        {
            let world = mob.get_entity().world.load();
            if let Some(target) = world.get_entity_by_id(target_id)
                && let Some(target_mob) = target.get_mob()
                && target
                    .get_entity()
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_ACCEPTS_IRON_GOLEM_GIFT)
                && target_mob
                    .get_mob_entity()
                    .is_slot_empty(&EQUIPMENT_SLOT_ANTENNA)
                && Self::golem_bounding_box(mob)
                    .intersects(&target.get_entity().bounding_box.load())
            {
                target_mob
                    .get_mob_entity()
                    .set_item_slot_and_drop_when_killed(
                        &EQUIPMENT_SLOT_ANTENNA,
                        ItemStack::new(1, &Item::POPPY),
                    );
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
