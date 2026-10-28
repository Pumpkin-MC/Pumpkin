use crate::entity::{EntityBase, ageable::AgeableMob};

use super::water_animal;

pub trait AgeableWaterCreature: AgeableMob {
    fn water_creature_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        water_animal::check_spawn_obstruction(self, world)
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32, caller: &dyn EntityBase) {
        water_animal::handle_air_supply(self, pre_tick_air_supply, caller);
    }

    fn water_creature_experience_reward(&self) -> u32 {
        water_animal::experience_reward(self)
    }
}
