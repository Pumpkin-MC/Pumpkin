use pumpkin_data::damage::DamageType;

use crate::entity::{
    EntityBase,
    breath::{DROWNING_DAMAGE, MAX_AIR},
    mob::Mob,
    passive::animal::experience_reward,
};

pub const AMBIENT_SOUND_INTERVAL: i32 = 120;

pub trait WaterAnimal: Mob {
    fn water_animal_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        check_spawn_obstruction(self, world)
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32, caller: &dyn EntityBase) {
        handle_air_supply(
            self,
            pre_tick_air_supply,
            caller,
            self.get_entity().is_in_water(),
            MAX_AIR,
            DROWNING_DAMAGE,
            DamageType::DROWN,
        );
    }

    fn water_animal_experience_reward(&self) -> u32 {
        experience_reward(self)
    }
}

pub(super) fn check_spawn_obstruction<M: Mob + ?Sized>(
    mob: &M,
    world: &crate::world::World,
) -> bool {
    world.is_unobstructed(mob.get_entity())
}

/// Runs out of air by one a tick unless `wet` (the water, or for the axolotl also rain) and
/// takes `dry_out_damage` of `dry_out_damage_type` once the air hits the drowning threshold.
pub(super) fn handle_air_supply<M: Mob + ?Sized>(
    mob: &M,
    pre_tick_air_supply: i32,
    caller: &dyn EntityBase,
    wet: bool,
    max_air_supply: i32,
    dry_out_damage: f32,
    dry_out_damage_type: DamageType,
) {
    let entity = mob.get_entity();
    let living = &mob.get_mob_entity().living_entity;
    if living.is_alive() && !wet {
        entity.set_air_supply(pre_tick_air_supply - 1);
        if living.should_take_drowning_damage() {
            entity.set_air_supply(0);
            caller.damage(caller, dry_out_damage, dry_out_damage_type);
        }
    } else {
        entity.set_air_supply(max_air_supply);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::abstract_fish::AbstractFish;
    use crate::entity::passive::cod::CodEntity;

    /// Fish keep the rule they had before it took parameters: 300 air in water, one less a tick on
    /// land, and a drowning hit of 2 once the air passes -20.
    #[tokio::test]
    async fn fish_keep_the_drowning_rule() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cod = CodEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COD,
        ));
        let entity = cod.get_entity();
        let living = &cod.mob_entity.living_entity;

        cod.fish_base_tick(cod.as_ref());
        let on_land = entity.air_supply.load(Relaxed);
        entity.air_supply.store(-19, Relaxed);
        cod.fish_base_tick(cod.as_ref());
        let dried_out = (
            entity.air_supply.load(Relaxed),
            living.get_last_damage_source(),
        );
        entity.touching_water.store(true, Relaxed);
        cod.fish_base_tick(cod.as_ref());
        let in_water = entity.air_supply.load(Relaxed);
        world.level.shutdown().await;

        assert_eq!(on_land, MAX_AIR - 1);
        assert_eq!(dried_out, (0, Some((DamageType::DROWN, 0))));
        assert_eq!(in_water, MAX_AIR);
        assert!((living.last_damage_taken.load() - DROWNING_DAMAGE).abs() < 1e-6);
        Ok(())
    }
}
