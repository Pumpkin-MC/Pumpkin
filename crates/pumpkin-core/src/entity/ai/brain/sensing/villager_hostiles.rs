use pumpkin_data::entity::EntityType;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 2] = [
    types::NEAREST_HOSTILE.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
];

const ACCEPTABLE_DISTANCE_FROM_HOSTILES: &[(&EntityType, f32)] = &[
    (&EntityType::DROWNED, 8.0),
    (&EntityType::EVOKER, 12.0),
    (&EntityType::HUSK, 8.0),
    (&EntityType::ILLUSIONER, 12.0),
    (&EntityType::PILLAGER, 15.0),
    (&EntityType::RAVAGER, 12.0),
    (&EntityType::VEX, 8.0),
    (&EntityType::VINDICATOR, 10.0),
    (&EntityType::ZOGLIN, 10.0),
    (&EntityType::ZOMBIE, 8.0),
    (&EntityType::ZOMBIE_VILLAGER, 8.0),
];

pub struct VillagerHostilesSensor;

fn hostile_distance(entity_type: &EntityType) -> Option<f32> {
    ACCEPTABLE_DISTANCE_FROM_HOSTILES
        .iter()
        .find_map(|(hostile, distance)| (*hostile == entity_type).then_some(*distance))
}

fn is_matching_entity(entity_type: &EntityType, squared_distance: f64) -> bool {
    hostile_distance(entity_type)
        .is_some_and(|distance| squared_distance <= f64::from(distance * distance))
}

impl Sensor for VillagerHostilesSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let position = tick.mob.get_entity().pos.load();
        let hostile = tick
            .state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
            .and_then(|visible| {
                visible.find_closest(&tick.visibility(), |entity| {
                    let entity = entity.get_entity();
                    is_matching_entity(
                        entity.entity_type,
                        position.squared_distance_to_vec(&entity.pos.load()),
                    )
                })
            });
        tick.state.set_optional(types::NEAREST_HOSTILE, hostile);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_distance_is_inclusive_and_ignores_other_types() {
        assert!(is_matching_entity(&EntityType::ZOMBIE, 64.0));
        assert!(!is_matching_entity(&EntityType::ZOMBIE, 64.000_001));
        assert!(!is_matching_entity(&EntityType::COW, 0.0));
    }
}
