use pumpkin_data::data_component::DataComponent;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::villager::VillagerProfession;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use super::super::utils;
use crate::entity::EntityBase;
use crate::entity::passive::villager::VillagerEntity;

const CLOSE_ENOUGH_DISTANCE: i32 = 2;
const SPEED_MODIFIER: f32 = 0.5;
const INTERACTION_DISTANCE_SQR: f64 = 5.0;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::INTERACTION_TARGET.id(), MemoryStatus::ValuePresent),
    (
        types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
        MemoryStatus::ValuePresent,
    ),
];

pub struct TradeWithVillager {
    trades: Vec<&'static Item>,
}

impl Default for TradeWithVillager {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeWithVillager {
    #[must_use]
    pub const fn new() -> Self {
        Self { trades: Vec::new() }
    }
}

impl Behavior for TradeWithVillager {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        utils::target_is_valid(&tick.visibility(), types::INTERACTION_TARGET, |target| {
            target.get_entity().entity_type == &EntityType::VILLAGER
        })
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.check_extra_start_conditions(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.state.get(types::INTERACTION_TARGET).cloned() else {
            return;
        };
        let Some(body) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        let Some(target_villager) = target.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &target,
            SPEED_MODIFIER,
            CLOSE_ENOUGH_DISTANCE,
        );
        self.trades = figure_out_what_i_am_willing_to_trade(body, target_villager);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.state.get(types::INTERACTION_TARGET).cloned() else {
            return;
        };
        let Some(body) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        let Some(target_villager) = target.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        if tick
            .mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load())
            > INTERACTION_DISTANCE_SQR
        {
            return;
        }
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &target,
            SPEED_MODIFIER,
            CLOSE_ENOUGH_DISTANCE,
        );
        body.gossip(tick.state, tick.world, target_villager, tick.time);
        let is_farmer = profession(body) == VillagerProfession::Farmer;
        if body.has_excess_food() && (is_farmer || target_villager.wants_more_food()) {
            throw_half_stack(tick, body, target.as_ref(), |stack| {
                stack.has_data_component(DataComponent::VillagerFood)
            });
        }
        if is_farmer {
            let max_stack_size = i32::from(ItemStack::new(1, &Item::WHEAT).get_max_stack_size());
            if body.inventory_count(&Item::WHEAT) > max_stack_size / 2 {
                throw_half_stack(tick, body, target.as_ref(), |stack| {
                    stack.item.id == Item::WHEAT.id
                });
            }
        }
        if !self.trades.is_empty() && has_any_of(body, &self.trades) {
            throw_half_stack(tick, body, target.as_ref(), |stack| {
                self.trades.iter().any(|trade| trade.id == stack.item.id)
            });
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::INTERACTION_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "TradeWithVillager"
    }
}

fn profession(villager: &VillagerEntity) -> VillagerProfession {
    villager
        .villager_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profession_enum()
}

fn figure_out_what_i_am_willing_to_trade(
    body: &VillagerEntity,
    target: &VillagerEntity,
) -> Vec<&'static Item> {
    let target_items = profession(target).requested_items();
    let self_items = profession(body).requested_items();
    target_items
        .iter()
        .copied()
        .filter(|item| !self_items.iter().any(|self_item| self_item.id == item.id))
        .collect()
}

fn has_any_of(villager: &VillagerEntity, items: &[&'static Item]) -> bool {
    villager
        .inventory
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .any(|stack| !stack.is_empty() && items.iter().any(|item| item.id == stack.item.id))
}

fn throw_half_stack(
    tick: &BrainTick<'_>,
    villager: &VillagerEntity,
    target: &dyn EntityBase,
    predicate: impl Fn(&ItemStack) -> bool,
) {
    let mut to_throw = ItemStack::EMPTY.clone();
    {
        let mut inventory = villager
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for item_stack in inventory.iter_mut() {
            if item_stack.is_empty() || !predicate(item_stack) {
                continue;
            }
            if let Some(count) =
                count_to_throw(item_stack.item_count, item_stack.get_max_stack_size())
            {
                to_throw = item_stack.split(count);
                break;
            }
        }
    }
    if !to_throw.is_empty() {
        utils::throw_item(tick.mob, to_throw, target.get_entity().pos.load());
    }
}

const fn count_to_throw(count: u8, max_stack_size: u8) -> Option<u8> {
    if count > max_stack_size / 2 {
        Some(count / 2)
    } else if count > 24 {
        Some(count - 24)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::count_to_throw;

    #[test]
    fn half_stack_count_follows_the_vanilla_thresholds() {
        assert_eq!(count_to_throw(33, 64), Some(16));
        assert_eq!(count_to_throw(32, 64), Some(8));
        assert_eq!(count_to_throw(25, 64), Some(1));
        assert_eq!(count_to_throw(24, 64), None);
        assert_eq!(count_to_throw(9, 16), Some(4));
        assert_eq!(count_to_throw(8, 16), None);
        assert_eq!(count_to_throw(0, 64), None);
    }
}
