use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::java::client::play::MerchantOffer;

use super::super::super::BrainTick;
use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use crate::entity::EntityBase;
use crate::entity::passive::villager::VillagerEntity;

const MAX_LOOK_TIME: i32 = 900;
const STARTING_LOOK_TIME: i32 = 40;
const ITEM_CYCLE_TICKS: i32 = 40;
const CLEARED_DROP_CHANCE: f32 = 0.085;
const DISPLAYED_DROP_CHANCE: f32 = 0.0;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 1] =
    [(types::INTERACTION_TARGET.id(), MemoryStatus::ValuePresent)];

pub struct ShowTradesToPlayer {
    player_item_stack: Option<ItemStack>,
    display_items: Vec<ItemStack>,
    cycle_counter: i32,
    display_index: usize,
    look_time: i32,
    min_duration: i32,
    max_duration: i32,
}

impl ShowTradesToPlayer {
    #[must_use]
    pub const fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            player_item_stack: None,
            display_items: Vec::new(),
            cycle_counter: 0,
            display_index: 0,
            look_time: 0,
            min_duration,
            max_duration,
        }
    }

    fn find_items_to_display(&mut self, target: &Arc<dyn EntityBase>, tick: &BrainTick<'_>) {
        let Some(player) = target.get_player() else {
            return;
        };
        let current = player.inventory.held_item();
        let changed = self
            .player_item_stack
            .as_ref()
            .is_none_or(|stack| !is_same_item(stack, &current));
        if changed {
            self.player_item_stack = Some(current.clone());
            self.display_items.clear();
        }
        if changed && !current.is_empty() {
            self.update_display_items(tick);
            if let Some(first) = self.display_items.first().cloned() {
                self.look_time = MAX_LOOK_TIME;
                display_as_held_item(tick, first);
            }
        }
    }

    fn update_display_items(&mut self, tick: &BrainTick<'_>) {
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        let player_item_stack = self.player_item_stack.clone();
        let Some(player_item_stack) = player_item_stack else {
            return;
        };
        let offers = villager
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for offer in offers.iter() {
            if !offer.is_out_of_stock()
                && player_item_stack_matches_cost_of_offer(&player_item_stack, offer)
            {
                self.display_items.push(offer.output.0.as_ref().clone());
            }
        }
    }

    fn display_cycling_items(&mut self, tick: &BrainTick<'_>) {
        if self.display_items.len() >= 2 {
            self.cycle_counter += 1;
            if self.cycle_counter >= ITEM_CYCLE_TICKS {
                self.display_index += 1;
                self.cycle_counter = 0;
                if self.display_index > self.display_items.len() - 1 {
                    self.display_index = 0;
                }
                let stack = self.display_items[self.display_index].clone();
                display_as_held_item(tick, stack);
            }
        }
    }
}

impl Behavior for ShowTradesToPlayer {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        self.min_duration
    }

    fn max_duration(&self) -> i32 {
        self.max_duration
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(target) = tick.state.get(types::INTERACTION_TARGET) else {
            return false;
        };
        let target = target.get_entity();
        target.entity_type == &EntityType::PLAYER
            && tick.mob.get_entity().is_alive()
            && target.is_alive()
            && !tick.mob.get_mob_entity().living_entity.is_baby()
            && tick
                .mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.pos.load())
                <= 17.0
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.check_extra_start_conditions(tick)
            && self.look_time > 0
            && tick.state.get(types::INTERACTION_TARGET).is_some()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        look_at_target(tick);
        self.cycle_counter = 0;
        self.display_index = 0;
        self.look_time = STARTING_LOOK_TIME;
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = look_at_target(tick) else {
            return;
        };
        self.find_items_to_display(&target, tick);
        if self.display_items.is_empty() {
            clear_held_item(tick);
            self.look_time = self.look_time.min(STARTING_LOOK_TIME);
        } else {
            self.display_cycling_items(tick);
        }
        self.look_time -= 1;
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::INTERACTION_TARGET.id());
        clear_held_item(tick);
        self.player_item_stack = None;
    }

    fn debug_name(&self) -> &'static str {
        "ShowTradesToPlayer"
    }
}

fn look_at_target(tick: &mut BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    let target = tick.state.get(types::INTERACTION_TARGET).cloned()?;
    tick.state.set(
        types::LOOK_TARGET,
        Arc::new(EntityTracker::new(target.clone(), true)) as Arc<dyn PositionTracker>,
    );
    Some(target)
}

const fn is_same_item(first: &ItemStack, second: &ItemStack) -> bool {
    first.item.id == second.item.id
}

fn player_item_stack_matches_cost_of_offer(
    player_item_stack: &ItemStack,
    offer: &MerchantOffer,
) -> bool {
    is_same_item(player_item_stack, offer.base_cost_a.0.as_ref())
        || offer
            .cost_b
            .as_ref()
            .is_some_and(|cost| is_same_item(player_item_stack, cost.0.as_ref()))
}

fn clear_held_item(tick: &BrainTick<'_>) {
    set_held_item(tick, ItemStack::EMPTY.clone(), CLEARED_DROP_CHANCE);
}

fn display_as_held_item(tick: &BrainTick<'_>, stack: ItemStack) {
    set_held_item(tick, stack, DISPLAYED_DROP_CHANCE);
}

fn set_held_item(tick: &BrainTick<'_>, stack: ItemStack, drop_chance: f32) {
    let mob_entity = tick.mob.get_mob_entity();
    mob_entity.set_item_slot(&EquipmentSlot::MAIN_HAND, stack);
    mob_entity
        .living_entity
        .equipment_drop_chances
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(EquipmentSlot::MAIN_HAND.clone(), drop_chance);
}
