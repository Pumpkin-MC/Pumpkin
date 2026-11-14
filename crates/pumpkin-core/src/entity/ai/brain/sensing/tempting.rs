use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::player::Player;

const REQUIRES: [MemoryModuleId; 1] = [types::TEMPTING_PLAYER.id()];

/// The nearest player holding an item the mob is tempted by.
pub struct TemptingSensor {
    temptations: fn(&dyn Mob, &ItemStack) -> bool,
}

impl TemptingSensor {
    #[must_use]
    pub const fn for_animal() -> Self {
        Self {
            temptations: |mob, stack| mob.as_animal().is_some_and(|animal| animal.is_food(stack)),
        }
    }

    #[must_use]
    pub const fn for_frog() -> Self {
        Self {
            temptations: |_, stack| stack.item.has_tag(&tag::Item::MINECRAFT_FROG_FOOD),
        }
    }
}

impl Sensor for TemptingSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let mob = tick.mob;
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
                    && ((self.temptations)(mob, &inventory.held_item())
                        || (self.temptations)(mob, &inventory.off_hand_item()))
                    && !mob.has_passenger(player.as_ref())
            })
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .cloned();
        tick.state.set_optional(types::TEMPTING_PLAYER, nearest);
    }
}
