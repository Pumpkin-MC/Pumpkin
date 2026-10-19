use std::sync::{Arc, PoisonError};

use pumpkin_data::{
    Block, BlockDirection, BlockState, data_component_impl::EquipmentSlot, dimension::Dimension,
    item_stack::ItemStack, tag,
};
use pumpkin_inventory::Inventory;
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector2::Vector2, vector3::Vector3,
};
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};

use crate::{
    block::{
        blocks::chests::{ChestContainer, get_chest_container},
        entities::BlockEntity,
    },
    entity::{
        mob::Mob,
        passive::copper_golem::{CopperGolemEntity, CopperGolemState},
    },
    plugin::api::events::inventory::inventory_move_item::InventoryMoveItemEvent,
    world::World,
};

use super::{Behavior, utils::set_walk_and_look_target_memories_to_block};
use crate::entity::ai::brain::{
    BrainTick,
    memory::{BlockPosTracker, GlobalPos, MemoryModuleId, MemoryStatus, types},
};

pub const TARGET_INTERACTION_TIME: i32 = 60;
const VISITED_POSITIONS_MEMORY_TIME: i64 = 6000;
const TRANSPORTED_ITEM_MAX_STACK_SIZE: u8 = 16;
const MAX_VISITED_POSITIONS: usize = 10;
const MAX_UNREACHABLE_POSITIONS: usize = 50;
const IDLE_COOLDOWN: i32 = 140;
const HORIZONTAL_SEARCH_DISTANCE: i32 = 32;
const VERTICAL_SEARCH_DISTANCE: i32 = 8;
const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 4] = [
    (
        types::VISITED_BLOCK_POSITIONS.id(),
        MemoryStatus::Registered,
    ),
    (
        types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS.id(),
        MemoryStatus::Registered,
    ),
    (
        types::TRANSPORT_ITEMS_COOLDOWN_TICKS.id(),
        MemoryStatus::ValueAbsent,
    ),
    (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
];

#[derive(Clone)]
struct TransportItemTarget {
    pos: BlockPos,
    container: ChestContainer,
    block_entity: Arc<dyn BlockEntity>,
    state: &'static BlockState,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum TransportItemState {
    #[default]
    Travelling,
    Queuing,
    Interacting,
}

#[derive(Default)]
pub struct TransportItemsBetweenContainers {
    target: Option<TransportItemTarget>,
    state: TransportItemState,
    interaction_state: Option<CopperGolemState>,
    ticks_since_reaching_target: i32,
}

impl TransportItemsBetweenContainers {
    fn is_picking_up_items(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().living_entity.held_item(mob).is_empty()
    }

    fn is_wanted_block(mob: &dyn Mob, state: &BlockState) -> bool {
        let block = Block::from_state_id(state.id);
        if Self::is_picking_up_items(mob) {
            block.id.has_tag(tag::Block::MINECRAFT_COPPER_CHESTS)
        } else {
            block == &Block::CHEST || block == &Block::TRAPPED_CHEST
        }
    }

    fn stop_targeting_current_target(&mut self, tick: &mut BrainTick<'_>) {
        self.ticks_since_reaching_target = 0;
        self.target = None;
        tick.mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stop();
        tick.brain.erase(types::WALK_TARGET.id());
        if let Some(golem) = tick.mob.cast_any().downcast_ref::<CopperGolemEntity>() {
            golem.close_container();
        }
    }

    fn on_start_travelling(&mut self, tick: &BrainTick<'_>) {
        if let Some(golem) = tick.mob.cast_any().downcast_ref::<CopperGolemEntity>() {
            golem.close_container();
            golem.set_state(CopperGolemState::Idle);
        }
        self.state = TransportItemState::Travelling;
        self.interaction_state = None;
        self.ticks_since_reaching_target = 0;
    }

    fn clear_memories_after_matching_target_found(&mut self, tick: &mut BrainTick<'_>) {
        self.stop_targeting_current_target(tick);
        tick.brain.erase(types::VISITED_BLOCK_POSITIONS.id());
        tick.brain
            .erase(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS.id());
    }

    fn enter_cooldown_after_no_matching_target_found(&mut self, tick: &mut BrainTick<'_>) {
        self.clear_memories_after_matching_target_found(tick);
        tick.brain
            .set(types::TRANSPORT_ITEMS_COOLDOWN_TICKS, IDLE_COOLDOWN);
    }

    fn global_pos(world: &World, pos: BlockPos) -> Option<GlobalPos> {
        Dimension::from_name(world.dimension.minecraft_name)
            .map(|dimension| GlobalPos::new(dimension, pos))
    }

    fn set_visited_block_pos(&mut self, tick: &mut BrainTick<'_>, pos: BlockPos) {
        let Some(pos) = Self::global_pos(tick.world, pos) else {
            return;
        };
        let mut visited = tick
            .brain
            .get(types::VISITED_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        visited.insert(pos);
        if visited.len() > MAX_VISITED_POSITIONS {
            self.enter_cooldown_after_no_matching_target_found(tick);
        } else {
            tick.brain.set_with_expiry(
                types::VISITED_BLOCK_POSITIONS,
                visited,
                VISITED_POSITIONS_MEMORY_TIME,
            );
        }
    }

    fn mark_visited_block_pos_as_unreachable(&mut self, tick: &mut BrainTick<'_>, pos: BlockPos) {
        let Some(pos) = Self::global_pos(tick.world, pos) else {
            return;
        };
        let mut visited = tick
            .brain
            .get(types::VISITED_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        let mut unreachable = tick
            .brain
            .get(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        visited.remove(&pos);
        unreachable.insert(pos);
        if unreachable.len() > MAX_UNREACHABLE_POSITIONS {
            self.enter_cooldown_after_no_matching_target_found(tick);
        } else {
            tick.brain.set_with_expiry(
                types::VISITED_BLOCK_POSITIONS,
                visited,
                VISITED_POSITIONS_MEMORY_TIME,
            );
            tick.brain.set_with_expiry(
                types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS,
                unreachable,
                VISITED_POSITIONS_MEMORY_TIME,
            );
        }
    }

    fn get_transport_target(tick: &BrainTick<'_>) -> Option<TransportItemTarget> {
        let entity = tick.mob.get_entity();
        let origin = entity.block_pos.load();
        let (horizontal, vertical) = if tick.mob.is_passenger() {
            (1, 1)
        } else {
            (HORIZONTAL_SEARCH_DISTANCE, VERTICAL_SEARCH_DISTANCE)
        };
        let center_chunk = origin.chunk_position();
        let chunk_radius = horizontal / 16 + 1;
        let visited = tick.brain.get(types::VISITED_BLOCK_POSITIONS);
        let unreachable = tick.brain.get(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS);
        let mut closest = f64::from(f32::MAX);
        let mut target = None;
        for x in center_chunk.x - chunk_radius..=center_chunk.x + chunk_radius {
            for z in center_chunk.y - chunk_radius..=center_chunk.y + chunk_radius {
                let chunk = Vector2::new(x, z);
                if !tick.world.level.is_chunk_loaded(&chunk) {
                    continue;
                }
                let entities: Vec<_> = tick
                    .world
                    .block_entities
                    .get(&chunk)
                    .map(|entry| entry.values().cloned().collect())
                    .unwrap_or_default();
                for block_entity in entities {
                    let pos = block_entity.get_position();
                    if (pos.0.x - origin.0.x).abs() > horizontal
                        || (pos.0.y - origin.0.y).abs() > vertical
                        || (pos.0.z - origin.0.z).abs() > horizontal
                    {
                        continue;
                    }
                    let distance = pos
                        .to_centered_f64()
                        .squared_distance_to_vec(&entity.pos.load());
                    if distance >= closest {
                        continue;
                    }
                    let state = tick.world.get_block_state(&pos);
                    if !Self::is_wanted_block(tick.mob, state) {
                        continue;
                    }
                    let Some(container) = get_chest_container(tick.world, &pos) else {
                        continue;
                    };
                    if container.positions.iter().any(|pos| {
                        Self::global_pos(tick.world, *pos).is_some_and(|pos| {
                            visited.is_some_and(|set| set.contains(&pos))
                                || unreachable.is_some_and(|set| set.contains(&pos))
                        })
                    }) {
                        continue;
                    }
                    target = Some(TransportItemTarget {
                        pos,
                        container,
                        block_entity,
                        state,
                    });
                    closest = distance;
                }
            }
        }
        target
    }

    fn middle_position(mob: &dyn Mob, bottom: Vector3<f64>) -> Vector3<f64> {
        let bounds = mob.get_entity().bounding_box.load();
        bottom + Vector3::new(0.0, (bounds.max.y - bounds.min.y) / 2.0, 0.0)
    }

    fn is_within_target_distance(
        distance: f64,
        target: &TransportItemTarget,
        mob: &dyn Mob,
        center: Vector3<f64>,
    ) -> bool {
        let entity = mob.get_entity();
        let bounds = entity.bounding_box.load();
        let moved = bounds.shift(center - Self::middle_position(mob, entity.pos.load()));
        let shape_bounds = target
            .state
            .get_block_collision_shapes_at(&target.pos)
            .reduce(|left, right| {
                BoundingBox::new(
                    Vector3::new(
                        left.min.x.min(right.min.x),
                        left.min.y.min(right.min.y),
                        left.min.z.min(right.min.z),
                    ),
                    Vector3::new(
                        left.max.x.max(right.max.x),
                        left.max.y.max(right.max.y),
                        left.max.z.max(right.max.z),
                    ),
                )
            });
        shape_bounds.is_some_and(|shape| {
            shape
                .expand(distance, 0.5, distance)
                .shift(target.pos.0.to_f64())
                .intersects(&moved)
        })
    }

    fn get_interaction_range(mob: &dyn Mob) -> f64 {
        if mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_path()
            .is_some_and(crate::entity::ai::pathfinder::path::Path::is_done)
        {
            1.0
        } else {
            0.5
        }
    }

    fn has_valid_target(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(target) = self.target.clone() else {
            return false;
        };
        if !Self::is_wanted_block(tick.mob, target.state)
            || !tick
                .world
                .get_block_entity(&target.pos)
                .is_some_and(|entity| Arc::ptr_eq(&entity, &target.block_entity))
            || !get_chest_container(tick.world, &target.pos)
                .is_some_and(|current| target.container.same_inventory(&current))
        {
            return false;
        }
        if self.state != TransportItemState::Travelling {
            return true;
        }
        let mob = tick.mob.get_mob_entity();
        let path = {
            let mut navigation = mob.navigator.lock().unwrap_or_else(PoisonError::into_inner);
            navigation
                .get_path()
                .cloned()
                .or_else(|| navigation.create_path(&mob.living_entity, target.pos.0.to_f64(), 0))
        };
        let bottom = path
            .as_ref()
            .and_then(|path| path.get_end_node())
            .map_or_else(
                || tick.mob.get_entity().pos.load(),
                |node| node.pos.to_centered_f64() - Vector3::new(0.0, 0.5, 0.0),
            );
        let center = Self::middle_position(tick.mob, bottom);
        let can_reach = Self::is_within_target_distance(
            Self::get_interaction_range(tick.mob),
            &target,
            tick.mob,
            center,
        );
        if path.is_none() && !can_reach {
            return true;
        }
        if can_reach
            && BlockDirection::all().iter().any(|direction| {
                let end = target.pos.to_centered_f64() + direction.to_offset().to_f64() * 0.5;
                let hit = tick.world.clip(&ClipContext::new(
                    center,
                    end,
                    ClipBlock::Collider,
                    ClipFluid::None,
                ));
                !hit.is_miss && hit.block_pos == target.pos
            })
        {
            return true;
        }
        self.mark_visited_block_pos_as_unreachable(tick, target.pos);
        false
    }

    fn walk_towards_target(tick: &mut BrainTick<'_>, pos: BlockPos) {
        set_walk_and_look_target_memories_to_block(tick.brain, pos, 1.0, 0);
    }

    fn stop_in_place(mob: &dyn Mob) {
        let mob = mob.get_mob_entity();
        mob.navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stop();
        mob.living_entity
            .movement_input
            .store(Vector3::new(0.0, 0.0, 0.0));
        let velocity = mob.living_entity.entity.velocity.load();
        mob.living_entity
            .entity
            .velocity
            .store(Vector3::new(0.0, velocity.y, 0.0));
    }

    fn interaction_state(mob: &dyn Mob, inventory: &dyn Inventory) -> CopperGolemState {
        if Self::is_picking_up_items(mob) {
            if inventory.is_empty() {
                CopperGolemState::GettingNoItem
            } else {
                CopperGolemState::GettingItem
            }
        } else if matches_leaving_items_requirement(
            &mob.get_mob_entity().living_entity.held_item(mob),
            inventory,
        ) {
            CopperGolemState::DroppingItem
        } else {
            CopperGolemState::DroppingNoItem
        }
    }

    fn transfer_allowed(
        tick: &BrainTick<'_>,
        target: &TransportItemTarget,
        stack: &ItemStack,
        count: u8,
        pickup: bool,
    ) -> bool {
        let entity_pos = tick.mob.get_entity().block_pos.load();
        let (source, destination) = if pickup {
            (target.pos, entity_pos)
        } else {
            (entity_pos, target.pos)
        };
        let mut event = InventoryMoveItemEvent::new(
            source,
            destination,
            stack.item.registry_key.to_owned(),
            u32::from(count),
        );
        if let Some(server) = tick.world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        !event.cancelled
            && tick.mob.get_mob_entity().living_entity.health.load() > 0.0
            && !tick
                .mob
                .get_entity()
                .removed
                .load(std::sync::atomic::Ordering::Relaxed)
            && get_chest_container(tick.world, &target.pos)
                .is_some_and(|current| target.container.same_inventory(&current))
    }

    fn do_reached_target_interaction(
        &mut self,
        tick: &mut BrainTick<'_>,
        target: &TransportItemTarget,
    ) {
        let inventory = target.container.inventory.as_ref();
        match Self::interaction_state(tick.mob, inventory) {
            CopperGolemState::GettingItem => {
                let first = (0..inventory.size())
                    .map(|slot| inventory.get_stack(slot))
                    .find(|stack| !stack.is_empty());
                if let Some(stack) = first
                    && Self::transfer_allowed(
                        tick,
                        target,
                        &stack,
                        stack.item_count.min(TRANSPORTED_ITEM_MAX_STACK_SIZE),
                        true,
                    )
                {
                    let mob = tick.mob.get_mob_entity();
                    // Death drops lock drop chances before equipment, so keep that order.
                    mob.set_guaranteed_drop(&EquipmentSlot::MAIN_HAND);
                    let picked = {
                        let mut equipment = mob
                            .living_entity
                            .entity_equipment
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);
                        (equipment.get(&EquipmentSlot::MAIN_HAND).is_empty()
                            && mob.living_entity.health.load() > 0.0
                            && !tick
                                .mob
                                .get_entity()
                                .removed
                                .load(std::sync::atomic::Ordering::Relaxed))
                        .then(|| {
                            let stack = pickup_item_from_container(&target.container, &stack);
                            equipment.put(&EquipmentSlot::MAIN_HAND, stack.clone());
                            stack
                        })
                    };
                    let Some(stack) = picked else {
                        self.stop_targeting_current_target(tick);
                        return;
                    };
                    mob.living_entity
                        .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
                    inventory.mark_dirty();
                    self.clear_memories_after_matching_target_found(tick);
                } else {
                    self.stop_targeting_current_target(tick);
                }
            }
            CopperGolemState::DroppingItem => {
                let stack = tick.mob.get_mob_entity().living_entity.held_item(tick.mob);
                let count = transferable_item_count(&stack, inventory);
                if count == 0 || !Self::transfer_allowed(tick, target, &stack, count, false) {
                    self.stop_targeting_current_target(tick);
                    return;
                }
                let mob = tick.mob.get_mob_entity();
                let remaining = {
                    let mut equipment = mob
                        .living_entity
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner);
                    let mut current = equipment.get(&EquipmentSlot::MAIN_HAND);
                    (current.are_items_and_components_equal(&stack)
                        && current.item_count == stack.item_count)
                        .then(|| {
                            let moving = current.split(count);
                            let rejected = add_items_to_container(moving, &target.container);
                            current.item_count += rejected.item_count;
                            let remaining = current;
                            equipment.put(&EquipmentSlot::MAIN_HAND, remaining.clone());
                            remaining
                        })
                };
                let Some(remaining) = remaining else {
                    self.stop_targeting_current_target(tick);
                    return;
                };
                let empty = remaining.is_empty();
                inventory.mark_dirty();
                mob.living_entity
                    .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, remaining)]);
                if empty {
                    self.clear_memories_after_matching_target_found(tick);
                } else {
                    self.stop_targeting_current_target(tick);
                }
            }
            _ => self.stop_targeting_current_target(tick),
        }
    }
}

impl Behavior for TransportItemsBetweenContainers {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        !tick.mob.get_entity().is_leashed()
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        !tick
            .brain
            .has_memory_value(types::TRANSPORT_ITEMS_COOLDOWN_TICKS.id())
            && !tick.brain.has_memory_value(types::IS_PANICKING.id())
            && !tick.mob.get_entity().is_leashed()
    }

    fn timed_out(&self, _time: i64, _end_timestamp: i64) -> bool {
        false
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .set_can_path_to_targets_below_surface(true);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if !self.has_valid_target(tick) {
            self.stop_targeting_current_target(tick);
            if let Some(target) = Self::get_transport_target(tick) {
                let pos = target.pos;
                self.target = Some(target);
                self.on_start_travelling(tick);
                self.set_visited_block_pos(tick, pos);
            } else {
                self.enter_cooldown_after_no_matching_target_found(tick);
            }
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        if self.state == TransportItemState::Queuing && !target.container.is_open(tick.world) {
            self.state = TransportItemState::Travelling;
            Self::walk_towards_target(tick, target.pos);
        }
        let center = Self::middle_position(tick.mob, tick.mob.get_entity().pos.load());
        if self.state == TransportItemState::Travelling {
            if Self::is_within_target_distance(3.0, &target, tick.mob, center)
                && target.container.is_open(tick.world)
            {
                Self::stop_in_place(tick.mob);
                self.state = TransportItemState::Queuing;
            } else if Self::is_within_target_distance(
                Self::get_interaction_range(tick.mob),
                &target,
                tick.mob,
                center,
            ) {
                target.container.unpack_loot(tick.world);
                self.interaction_state = Some(Self::interaction_state(
                    tick.mob,
                    target.container.inventory.as_ref(),
                ));
                self.state = TransportItemState::Interacting;
            } else {
                Self::walk_towards_target(tick, target.pos);
            }
        }
        if self.state == TransportItemState::Interacting {
            if !Self::is_within_target_distance(2.0, &target, tick.mob, center) {
                self.on_start_travelling(tick);
                return;
            }
            self.ticks_since_reaching_target += 1;
            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(BlockPosTracker::new(target.pos)),
            );
            Self::stop_in_place(tick.mob);
            if let Some(golem) = tick.mob.cast_any().downcast_ref::<CopperGolemEntity>()
                && !golem.on_container_interaction(
                    &target.container,
                    self.interaction_state.unwrap_or(CopperGolemState::Idle),
                    self.ticks_since_reaching_target,
                )
            {
                self.ticks_since_reaching_target = 0;
                self.state = TransportItemState::Queuing;
                return;
            }
            if self.ticks_since_reaching_target >= TARGET_INTERACTION_TIME {
                self.do_reached_target_interaction(tick, &target);
                self.on_start_travelling(tick);
            }
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        self.on_start_travelling(tick);
        tick.mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .set_can_path_to_targets_below_surface(false);
    }

    fn debug_name(&self) -> &'static str {
        "TransportItemsBetweenContainers"
    }
}

fn matches_leaving_items_requirement(stack: &ItemStack, inventory: &dyn Inventory) -> bool {
    inventory.is_empty()
        || (0..inventory.size()).any(|slot| {
            let candidate = inventory.get_stack(slot);
            !candidate.is_empty() && candidate.item.id == stack.item.id
        })
}

fn pickup_item_from_container(container: &ChestContainer, expected: &ItemStack) -> ItemStack {
    for slot in 0..container.inventory.size() {
        if let Some(stack) = container.modify_stack(slot, |stack| {
            if stack.is_empty() || !stack.are_items_and_components_equal(expected) {
                return ItemStack::EMPTY.clone();
            }
            stack.split(expected.item_count.min(TRANSPORTED_ITEM_MAX_STACK_SIZE))
        }) && !stack.is_empty()
        {
            return stack;
        }
    }
    ItemStack::EMPTY.clone()
}

fn add_items_to_container(mut stack: ItemStack, container: &ChestContainer) -> ItemStack {
    for slot in 0..container.inventory.size() {
        container.modify_stack(slot, |existing| {
            if existing.is_empty() {
                *existing = stack.split(stack.item_count.min(stack.get_max_stack_size()));
            } else if existing.are_items_and_components_equal(&stack) {
                let count = existing
                    .get_max_stack_size()
                    .saturating_sub(existing.item_count)
                    .min(stack.item_count);
                existing.item_count += count;
                stack.item_count -= count;
            }
        });
        if stack.is_empty() {
            return ItemStack::EMPTY.clone();
        }
    }
    stack
}

fn transferable_item_count(stack: &ItemStack, inventory: &dyn Inventory) -> u8 {
    let mut available: u8 = 0;
    for slot in 0..inventory.size() {
        let existing = inventory.get_stack(slot);
        let space = if existing.is_empty() {
            stack.get_max_stack_size()
        } else if existing.are_items_and_components_equal(stack) {
            existing
                .get_max_stack_size()
                .saturating_sub(existing.item_count)
        } else {
            0
        };
        available = available.saturating_add(space).min(stack.item_count);
        if available == stack.item_count {
            break;
        }
    }
    available
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::entities::chest::ChestBlockEntity;
    use pumpkin_data::item::Item;

    #[test]
    fn depleted_matching_stack_does_not_allow_deposit_into_a_nonmatching_chest() {
        let destination = ChestBlockEntity::new(BlockPos::new(0, 0, 0));
        destination.set_stack(0, ItemStack::new(0, &Item::IRON_INGOT));
        destination.set_stack(1, ItemStack::new(9, &Item::STONE));
        assert!(!matches_leaving_items_requirement(
            &ItemStack::new(16, &Item::IRON_INGOT),
            &destination
        ));
    }

    #[test]
    fn transport_preserves_items_when_the_destination_only_has_partial_space() {
        let source = Arc::new(ChestBlockEntity::new(BlockPos::new(0, 0, 0)));
        source.set_stack(1, ItemStack::new(63, &Item::IRON_INGOT));
        source.set_stack(2, ItemStack::new(64, &Item::STONE));
        let source_container = ChestContainer::single(source.position, source.clone());
        let carried = pickup_item_from_container(&source_container, &source.get_stack(1));
        assert_eq!(carried.item_count, 16);
        assert_eq!(source.get_stack(1).item_count, 47);
        assert_eq!(source.get_stack(2).item_count, 64);

        let destination = Arc::new(ChestBlockEntity::new(BlockPos::new(2, 0, 0)));
        destination.set_stack(0, ItemStack::new(60, &Item::IRON_INGOT));
        for slot in 1..destination.size() {
            destination.set_stack(slot, ItemStack::new(64, &Item::STONE));
        }
        let destination_container =
            ChestContainer::single(destination.position, destination.clone());
        assert_eq!(transferable_item_count(&carried, destination.as_ref()), 4);
        let remaining = add_items_to_container(carried, &destination_container);
        assert_eq!(destination.get_stack(0).item_count, 64);
        assert_eq!(remaining.item_count, 12);
        assert_eq!(remaining.item.id, Item::IRON_INGOT.id);
        assert_eq!(transferable_item_count(&remaining, destination.as_ref()), 0);

        destination.set_stack(1, ItemStack::EMPTY.clone());
        assert!(add_items_to_container(remaining, &destination_container).is_empty());
        assert_eq!(destination.get_stack(1).item_count, 12);
        assert_eq!(destination.get_stack(2).item.id, Item::STONE.id);
    }
}
