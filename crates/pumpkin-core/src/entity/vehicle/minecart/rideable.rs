use std::sync::Arc;

use crate::entity::{Entity, EntityBase, player::Player};

pub(super) struct RideableMinecart;

impl RideableMinecart {
    pub(super) fn interact(entity: &Entity, player: &Arc<Player>) -> bool {
        if player.get_entity().is_sneaking()
            || !entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
            || player.get_entity().has_vehicle()
        {
            return false;
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
