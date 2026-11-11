use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::{EntityPose, EntityType};
use rand::RngExt;

use super::data::{BREEDING_FOOD_THRESHOLD, VillagerData, VillagerProfession, VillagerType};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;
use crate::world::World;

const EXCESS_FOOD_THRESHOLD: i32 = 24;
const DIGEST_FOOD_AMOUNT: i32 = 12;
const BIOME_ROLL_OWN_TYPE: f64 = 0.5;
const BIOME_ROLL_PARTNER_TYPE: f64 = 0.75;

const fn can_breed_with_food(food_level: i32, food_points: i32) -> bool {
    food_level + food_points >= BREEDING_FOOD_THRESHOLD
}

const fn breed_type_roll(
    roll: f64,
    biome_type: VillagerType,
    own_type: VillagerType,
    partner_type: VillagerType,
) -> VillagerType {
    if roll < BIOME_ROLL_OWN_TYPE {
        biome_type
    } else if roll < BIOME_ROLL_PARTNER_TYPE {
        own_type
    } else {
        partner_type
    }
}

pub(super) fn villager_type_for_biome(registry_id: &str) -> VillagerType {
    match registry_id {
        "badlands" | "desert" | "eroded_badlands" | "wooded_badlands" => VillagerType::Desert,
        "bamboo_jungle" | "jungle" | "sparse_jungle" => VillagerType::Jungle,
        "savanna" | "savanna_plateau" | "windswept_savanna" => VillagerType::Savanna,
        "deep_frozen_ocean" | "frozen_ocean" | "frozen_river" | "frozen_peaks" | "grove"
        | "ice_spikes" | "jagged_peaks" | "snowy_beach" | "snowy_plains" | "snowy_slopes"
        | "snowy_taiga" => VillagerType::Snow,
        "mangrove_swamp" | "swamp" => VillagerType::Swamp,
        "old_growth_pine_taiga"
        | "old_growth_spruce_taiga"
        | "taiga"
        | "windswept_forest"
        | "windswept_gravelly_hills"
        | "windswept_hills" => VillagerType::Taiga,
        _ => VillagerType::Plains,
    }
}

impl VillagerEntity {
    pub fn can_breed(&self) -> bool {
        can_breed_with_food(
            self.food_level.load(Relaxed),
            self.count_food_points_in_inventory(),
        ) && self.get_entity().pose.load() != EntityPose::Sleeping
            && self.get_entity().age.load(Relaxed) == 0
    }

    pub fn eat_and_digest_food(&self) {
        self.eat_until_full();
        self.food_level.fetch_sub(DIGEST_FOOD_AMOUNT, Relaxed);
    }

    pub fn has_excess_food(&self) -> bool {
        self.count_food_points_in_inventory() >= EXCESS_FOOD_THRESHOLD
    }

    pub fn wants_more_food(&self) -> bool {
        self.count_food_points_in_inventory() < BREEDING_FOOD_THRESHOLD
    }

    pub fn get_breed_offspring(&self, world: &Arc<World>, partner: &Self) -> Option<Arc<Self>> {
        let biome_type = villager_type_for_biome(
            world
                .get_biome(&self.get_entity().block_pos.load())
                .registry_id,
        );
        let own_type = self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .type_enum();
        let partner_type = partner
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .type_enum();
        let r#type = breed_type_roll(
            self.get_random().random::<f64>(),
            biome_type,
            own_type,
            partner_type,
        );

        let baby = Self::new(Entity::new(
            world.clone(),
            self.get_entity().pos.load(),
            &EntityType::VILLAGER,
        ));
        baby.set_villager_data(VillagerData::new(r#type, VillagerProfession::None, 1));
        baby.set_villager_data_finalized(true);
        Some(baby)
    }
}

#[cfg(test)]
mod tests {
    use super::{breed_type_roll, can_breed_with_food};
    use crate::entity::passive::villager::VillagerType;

    #[test]
    fn can_breed_needs_twelve_food_points() {
        assert!(!can_breed_with_food(0, 11));
        assert!(can_breed_with_food(0, 12));
        assert!(can_breed_with_food(5, 7));
        assert!(can_breed_with_food(12, 0));
        assert!(!can_breed_with_food(5, 6));
    }

    #[test]
    fn breed_type_roll_uses_biome_own_and_partner_ranges() {
        let biome_type = VillagerType::Desert;
        let own_type = VillagerType::Jungle;
        let partner_type = VillagerType::Snow;

        assert_eq!(
            breed_type_roll(0.0, biome_type, own_type, partner_type),
            biome_type
        );
        assert_eq!(
            breed_type_roll(0.499, biome_type, own_type, partner_type),
            biome_type
        );
        assert_eq!(
            breed_type_roll(0.5, biome_type, own_type, partner_type),
            own_type
        );
        assert_eq!(
            breed_type_roll(0.749, biome_type, own_type, partner_type),
            own_type
        );
        assert_eq!(
            breed_type_roll(0.75, biome_type, own_type, partner_type),
            partner_type
        );
        assert_eq!(
            breed_type_roll(1.0, biome_type, own_type, partner_type),
            partner_type
        );
    }
}
