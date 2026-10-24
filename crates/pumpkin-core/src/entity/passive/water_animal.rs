use pumpkin_data::damage::DamageType;
use rand::RngExt;

use crate::entity::{
    EntityBase,
    breath::{DROWNING_DAMAGE, DROWNING_INTERVAL, MAX_AIR},
    mob::Mob,
};

pub const AMBIENT_SOUND_INTERVAL: i32 = 120;

pub trait WaterAnimal: Mob {
    fn water_animal_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        let entity = self.get_entity();
        let bounds = entity.bounding_box.load();
        let root_vehicle = entity.root_vehicle_id();
        let unobstructed = |other: &dyn EntityBase| {
            let other = other.get_entity();
            !other.is_alive() || !other.blocks_building || other.root_vehicle_id() == root_vehicle
        };
        world
            .get_entities_at_box(&bounds)
            .iter()
            .all(|other| unobstructed(other.as_ref()))
            && world
                .get_players_at_box(&bounds)
                .iter()
                .all(|player| player.is_spectator() || unobstructed(player.as_ref()))
    }

    fn handle_air_supply(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let living = &self.get_mob_entity().living_entity;
        if !living.is_dead_or_dying() && entity.is_alive() && !entity.is_in_water() {
            entity.set_air_supply(entity.air_supply.load(std::sync::atomic::Ordering::Relaxed) - 1);
            if entity.air_supply.load(std::sync::atomic::Ordering::Relaxed) <= -DROWNING_INTERVAL {
                entity.set_air_supply(0);
                caller.damage(caller, DROWNING_DAMAGE, DamageType::DROWN);
            }
        } else {
            entity.set_air_supply(MAX_AIR);
        }
    }

    fn water_animal_experience_reward(&self) -> u32 {
        1 + self.get_random().random_range(0..3)
    }
}
