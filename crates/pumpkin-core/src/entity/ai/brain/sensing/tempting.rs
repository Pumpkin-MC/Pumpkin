use std::sync::Arc;

use pumpkin_data::attributes::Attributes;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::player::Player;

const REQUIRES: [MemoryModuleId; 1] = [types::TEMPTING_PLAYER.id()];

/// The nearest player holding food the animal eats.
pub struct FoodTemptationSensor;

impl Sensor for FoodTemptationSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let mob = tick.mob;
        let Some(animal) = mob.as_animal() else {
            return;
        };
        let living = &mob.get_mob_entity().living_entity;
        let targeting = TargetPredicate::create_non_attackable()
            .ignore_visibility()
            .set_base_max_distance(living.get_attribute_value(&Attributes::TEMPT_RANGE));
        let position = living.entity.pos.load();
        let distance = |player: &Arc<Player>| {
            position.squared_distance_to_vec(&player.get_entity().pos.load())
        };
        let nearest = tick
            .world
            .players
            .load()
            .iter()
            .filter(|player| {
                let inventory = player.inventory();
                !player.is_spectator()
                    && targeting.test(tick.world, Some(mob), player.as_ref())
                    && (animal.is_food(&inventory.held_item())
                        || animal.is_food(&inventory.off_hand_item()))
                    && !mob.has_passenger(player.as_ref())
            })
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .cloned();
        tick.state.set_optional(types::TEMPTING_PLAYER, nearest);
    }
}
