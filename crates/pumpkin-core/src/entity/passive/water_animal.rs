use pumpkin_data::damage::DamageType;
use rand::RngExt;

use crate::entity::{
    EntityBase,
    breath::{DROWNING_DAMAGE, MAX_AIR},
    mob::Mob,
};

pub const AMBIENT_SOUND_INTERVAL: i32 = 120;

pub trait WaterAnimal: Mob {
    fn water_animal_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        check_spawn_obstruction(self, world)
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32, caller: &dyn EntityBase) {
        handle_air_supply(self, pre_tick_air_supply, caller);
    }

    fn water_animal_experience_reward(&self) -> u32 {
        experience_reward(self)
    }
}

pub(super) fn check_spawn_obstruction<M: Mob + ?Sized>(
    mob: &M,
    world: &crate::world::World,
) -> bool {
    let entity = mob.get_entity();
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

pub(super) fn handle_air_supply<M: Mob + ?Sized>(
    mob: &M,
    pre_tick_air_supply: i32,
    caller: &dyn EntityBase,
) {
    let entity = mob.get_entity();
    let living = &mob.get_mob_entity().living_entity;
    if living.is_alive() && !entity.is_in_water() {
        entity.set_air_supply(pre_tick_air_supply - 1);
        if living.should_take_drowning_damage() {
            entity.set_air_supply(0);
            caller.damage(caller, DROWNING_DAMAGE, DamageType::DROWN);
        }
    } else {
        entity.set_air_supply(MAX_AIR);
    }
}

pub(super) fn experience_reward<M: Mob + ?Sized>(mob: &M) -> u32 {
    1 + mob.get_random().random_range(0..3)
}
