use std::sync::Arc;

use pumpkin_data::sound::Sound;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity},
    passive::fish::{self, FishAir},
};

/// Represents a Pufferfish, a passive aquatic mob that can inflate when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Pufferfish>
pub struct PufferfishEntity {
    pub mob_entity: MobEntity,
    air: FishAir,
}

impl PufferfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let fish = Self {
            mob_entity: MobEntity::new(entity),
            air: FishAir::default(),
        };
        fish::init(&fish.mob_entity);
        Arc::new(fish)
    }
}

impl Mob for PufferfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        fish::tick(self, caller, &self.air, Sound::EntityPufferFishFlop);
    }

    fn mob_travel_in_water(&self, caller: &dyn EntityBase) -> bool {
        fish::travel_in_water(self, caller)
    }

    // Vanilla `WaterAnimal`: currents don't carry fish.
    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }
}
