use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};

use crate::entity::{Entity, EntityBase, living::is_allowed_by_team_rules, player::Player};

pub(super) struct RideableMinecart;

impl RideableMinecart {
    pub(super) fn pick_up_nearby_mob(entity: &Entity, still_current: impl FnOnce() -> bool) {
        if entity.has_passengers() {
            return;
        }
        let world = entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
            return;
        };
        let bounds = entity.bounding_box.load().expand(0.2, 0.0, 0.2);
        let can_pick_up = |other: &dyn EntityBase| {
            let target = other.get_entity();
            target.entity_id != entity.entity_id
                && !target.is_removed()
                && target.entity_type != &EntityType::IRON_GOLEM
                && !target.entity_type.has_tag(&tag::EntityType::C_MINECARTS)
                && !other.is_passenger()
                && other.is_pushable()
                && target.riding_cooldown.load(Ordering::Relaxed) == 0
                && is_allowed_by_team_rules(vehicle.get_team().as_ref(), other.get_team().as_ref())
                && other.can_ride(vehicle.as_ref())
        };
        if let Some(passenger) = world
            .get_entities_at_box(&bounds)
            .into_iter()
            .find(|other| can_pick_up(other.as_ref()))
        {
            entity.add_passenger_if(vehicle.clone(), passenger.clone(), 1, || {
                still_current()
                    && can_pick_up(passenger.as_ref())
                    && bounds.intersects(&passenger.get_entity().bounding_box.load())
            });
        }
    }

    pub(super) fn interact(entity: &Entity, player: &Arc<Player>) -> bool {
        entity.mount_vehicle_by_interaction(player, 1, || !entity.has_passengers())
    }
}
