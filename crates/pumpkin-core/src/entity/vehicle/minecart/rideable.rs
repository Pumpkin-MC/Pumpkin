use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::{Entity, EntityBase, player::Player};

pub(super) struct RideableMinecart;

impl RideableMinecart {
    /// Vanilla `Minecart.interact`, boarding through `Entity.startRiding`.
    pub(super) fn interact(entity: &Entity, player: &Arc<Player>) -> bool {
        let rider = player.get_entity();
        if rider.is_sneaking() || entity.has_passengers() {
            return false;
        }
        // `canRide`, then a rider of another vehicle leaves it first. A failed boarding
        // still consumes the click.
        if rider.riding_cooldown.load(Ordering::Relaxed) > 0 {
            return true;
        }
        if let Some(old_vehicle) = rider.get_vehicle() {
            old_vehicle.get_entity().remove_passenger(rider.entity_id);
            if rider.has_vehicle() {
                return true;
            }
        }

        let world = entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
            return false;
        };
        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        entity.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
        true
    }
}

impl RideableMinecart {
    /// Vanilla `ejectPassengers`: last rider first.
    pub(super) fn eject_passengers(entity: &Entity) {
        let riders: Vec<i32> = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .rev()
            .map(|passenger| passenger.get_entity().entity_id)
            .collect();
        for rider in riders {
            entity.remove_passenger(rider);
        }
    }
}
