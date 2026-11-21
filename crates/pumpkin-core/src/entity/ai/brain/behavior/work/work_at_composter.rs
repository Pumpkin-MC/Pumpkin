use std::sync::Arc;

use pumpkin_data::block_properties::ComposterLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;

use super::super::super::BrainTick;
use super::super::super::memory::types;
use super::work_at_poi::WorkAtPoi;
use crate::block::blocks::composter::ComposterBlock;
use crate::entity::EntityBase;
use crate::entity::passive::villager::VillagerEntity;

const COMPOSTABLE_ITEMS: [&Item; 2] = [&Item::WHEAT_SEEDS, &Item::BEETROOT_SEEDS];

const BREAD_CAP: i32 = 36;
const MAX_AMOUNT_OF_BREAD_TO_MAKE: i32 = 3;
const AMOUNT_OF_WHEAT_NEEDED_TO_CRAFT_ONE_BREAD: i32 = 3;

const TOTAL_ITEMS_TO_USE: i32 = 20;
const MIN_STACK_SIZE: i32 = 10;

#[must_use]
pub fn create() -> WorkAtPoi {
    WorkAtPoi::with_workstation(use_workstation)
}

fn use_workstation(tick: &mut BrainTick<'_>, villager: &VillagerEntity) {
    let Some(job_site) = tick.state.get(types::JOB_SITE).copied() else {
        return;
    };
    let (block, state) = tick.world.get_block_and_state(&job_site.pos);
    if block == &Block::COMPOSTER {
        make_bread(villager);
        compost_items(tick.world, villager, job_site.pos, state.id);
    }
}

fn make_bread(villager: &VillagerEntity) {
    let bread_count = villager.inventory_count(&Item::BREAD);
    let wheat_count = villager.inventory_count(&Item::WHEAT);
    let Some((bread_to_make, wheat_to_use)) = bread_to_make(wheat_count, bread_count) else {
        return;
    };
    let mut inventory = villager
        .inventory
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    remove_item_type(&mut inventory, &Item::WHEAT, wheat_to_use);
    let leftover = add_item(
        inventory.as_mut_slice(),
        ItemStack::new(bread_to_make, &Item::BREAD),
    );
    drop(inventory);
    if !leftover.is_empty() {
        villager.get_entity().spawn_at_location(leftover);
    }
}

fn bread_to_make(wheat_count: i32, bread_count: i32) -> Option<(u8, i32)> {
    if bread_count > BREAD_CAP {
        return None;
    }
    let bread_to_make =
        MAX_AMOUNT_OF_BREAD_TO_MAKE.min(wheat_count / AMOUNT_OF_WHEAT_NEEDED_TO_CRAFT_ONE_BREAD);
    if bread_to_make == 0 {
        return None;
    }
    Some((
        bread_to_make as u8,
        bread_to_make * AMOUNT_OF_WHEAT_NEEDED_TO_CRAFT_ONE_BREAD,
    ))
}

fn compost_items(
    world: &Arc<crate::world::World>,
    villager: &VillagerEntity,
    pos: BlockPos,
    state_id: BlockStateId,
) {
    let mut state_id = state_id;
    if ComposterLikeProperties::from_state_id(state_id).level == 8 {
        state_id = ComposterBlock::extract_produce(Some(villager), state_id, world, &pos);
    }

    let original_state_id = state_id;
    let mut total_items_to_use = TOTAL_ITEMS_TO_USE;
    let mut items_seen_so_far = [0i32; COMPOSTABLE_ITEMS.len()];
    let mut inventory = villager
        .inventory
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let container_size = inventory.len();
    let mut temp_state_id = state_id;

    for i in (0..container_size).rev() {
        if total_items_to_use <= 0 {
            break;
        }
        let item_stack = &mut inventory[i];
        let Some(item_index) = COMPOSTABLE_ITEMS
            .iter()
            .position(|item| item.id == item_stack.item.id)
        else {
            continue;
        };
        let stack_size = i32::from(item_stack.item_count);
        let total_item_count = items_seen_so_far[item_index] + stack_size;
        items_seen_so_far[item_index] = total_item_count;
        let items_to_use = items_to_use(total_item_count, total_items_to_use, stack_size);
        if items_to_use > 0 {
            total_items_to_use -= items_to_use;
            for _ in 0..items_to_use {
                temp_state_id = ComposterBlock::insert_item(
                    Some(villager),
                    temp_state_id,
                    world,
                    item_stack,
                    &pos,
                );
                if ComposterLikeProperties::from_state_id(temp_state_id).level == 7 {
                    spawn_composter_fill_effects(world, &pos, original_state_id, temp_state_id);
                    return;
                }
            }
        }
    }

    spawn_composter_fill_effects(world, &pos, original_state_id, temp_state_id);
}

fn items_to_use(total_item_count: i32, total_items_to_use: i32, stack_size: i32) -> i32 {
    (total_item_count - MIN_STACK_SIZE)
        .min(total_items_to_use)
        .min(stack_size)
}

fn spawn_composter_fill_effects(
    world: &Arc<crate::world::World>,
    pos: &BlockPos,
    original_state_id: BlockStateId,
    new_state_id: BlockStateId,
) {
    world.sync_world_event(
        WorldEvent::ComposterFill,
        *pos,
        i32::from(new_state_id != original_state_id),
    );
}

fn remove_item_type(inventory: &mut [ItemStack], item: &'static Item, count: i32) {
    let mut removed = 0;
    for slot in inventory.iter_mut().rev() {
        if slot.item.id == item.id {
            let still_needed = count - removed;
            if still_needed <= 0 {
                break;
            }
            let taken = slot
                .item_count
                .min(u8::try_from(still_needed).unwrap_or(u8::MAX));
            slot.decrement(taken);
            if slot.item_count == 0 {
                slot.clear();
            }
            removed += i32::from(taken);
            if removed == count {
                break;
            }
        }
    }
}

fn add_item(inventory: &mut [ItemStack], mut stack: ItemStack) -> ItemStack {
    if stack.is_empty() {
        return stack;
    }
    for slot in inventory.iter_mut() {
        if stack.is_empty() {
            return stack;
        }
        if !slot.is_empty() && slot.are_items_and_components_equal(&stack) {
            let count = stack
                .item_count
                .min(slot.get_max_stack_size().saturating_sub(slot.item_count));
            stack.decrement(count);
            slot.increment(count);
        }
    }
    for slot in inventory.iter_mut() {
        if stack.is_empty() {
            return stack;
        }
        if slot.is_empty() {
            *slot = std::mem::replace(&mut stack, ItemStack::EMPTY.clone());
        }
    }
    stack
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bread_needs_three_wheat_and_respects_the_cap() {
        assert_eq!(bread_to_make(8, 0), Some((2, 6)));
        assert_eq!(bread_to_make(2, 0), None);
        assert_eq!(bread_to_make(9, 36), Some((3, 9)));
        assert_eq!(bread_to_make(30, 37), None);
        assert_eq!(bread_to_make(30, 36), Some((3, 9)));
    }

    #[test]
    fn items_to_use_keeps_ten_in_the_stack_and_stops_at_twenty() {
        assert_eq!(items_to_use(5, TOTAL_ITEMS_TO_USE, 10), -5);
        assert_eq!(items_to_use(10, TOTAL_ITEMS_TO_USE, 10), 0);
        assert_eq!(items_to_use(15, TOTAL_ITEMS_TO_USE, 10), 5);
        assert_eq!(items_to_use(30, TOTAL_ITEMS_TO_USE, 10), 10);
        assert_eq!(items_to_use(30, 3, 10), 3);
        assert_eq!(items_to_use(30, TOTAL_ITEMS_TO_USE, 2), 2);
    }
}
