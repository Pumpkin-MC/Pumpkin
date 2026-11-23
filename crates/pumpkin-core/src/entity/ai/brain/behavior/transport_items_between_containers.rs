use std::sync::Arc;

use pumpkin_data::block_properties::{ChestLikeProperties, ChestType};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_inventory::Inventory;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};
use rustc_hash::FxHashSet;

use crate::block::blocks::chests;
use crate::block::entities::BlockEntity;
use crate::block::entities::chest::ChestBlockEntity;
use crate::block::entities::trapped_chest::TrappedChestBlockEntity;
use crate::entity::mob::Mob;
use crate::world::World;

use super::super::BrainTick;
use super::super::memory::position_tracker::BlockPosTracker;
use super::super::memory::{GlobalPos, MemoryModuleId, MemoryStatus, types};
use super::{BehaviorControl, Status, utils};

pub const TARGET_INTERACTION_TIME: i32 = 60;
const VISITED_POSITIONS_MEMORY_TIME: i64 = 6000;
const TRANSPORTED_ITEM_MAX_STACK_SIZE: u8 = 16;
const MAX_VISITED_POSITIONS: usize = 10;
const MAX_UNREACHABLE_POSITIONS: usize = 50;
const PASSENGER_MOB_TARGET_SEARCH_DISTANCE: i32 = 1;
const IDLE_COOLDOWN: i32 = 140;
const CLOSE_ENOUGH_TO_START_QUEUING_DISTANCE: f64 = 3.0;
const CLOSE_ENOUGH_TO_START_INTERACTING_WITH_TARGET_DISTANCE: f64 = 0.5;
const CLOSE_ENOUGH_TO_START_INTERACTING_WITH_TARGET_PATH_END_DISTANCE: f64 = 1.0;
const CLOSE_ENOUGH_TO_CONTINUE_INTERACTING_WITH_TARGET: f64 = 2.0;

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

const REQUIRED_MEMORIES: [MemoryModuleId; 4] = [
    types::VISITED_BLOCK_POSITIONS.id(),
    types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS.id(),
    types::TRANSPORT_ITEMS_COOLDOWN_TICKS.id(),
    types::IS_PANICKING.id(),
];

const SIDE_OFFSETS: [(f64, f64, f64); 6] = [
    (0.0, -0.5, 0.0),
    (0.0, 0.5, 0.0),
    (0.0, 0.0, -0.5),
    (0.0, 0.0, 0.5),
    (-0.5, 0.0, 0.0),
    (0.5, 0.0, 0.0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerInteractionState {
    PickupItem,
    PickupNoItem,
    PlaceItem,
    PlaceNoItem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransportItemState {
    Travelling,
    Queuing,
    Interacting,
}

pub struct TransportItemTarget {
    pub pos: BlockPos,
    pub block_entity: Arc<dyn BlockEntity>,
    pub block: &'static Block,
    pub state_id: BlockStateId,
}

impl TransportItemTarget {
    fn try_create(block_entity: Arc<dyn BlockEntity>, world: &World) -> Option<Self> {
        let pos = block_entity.get_position();
        let (block, state_id) = world.get_block_and_state_id(&pos);
        Arc::clone(&block_entity).get_container()?;
        Some(Self {
            pos,
            block_entity,
            block,
            state_id,
        })
    }

    /// The chest's inventory, joined with its current other half. Looked up when it is used,
    /// so finding a target never unpacks loot and a broken half is never written to.
    pub fn container(&self, world: &World) -> Option<Arc<dyn Inventory>> {
        let (block, state_id) = world.get_block_and_state_id(&self.pos);
        chests::chest_container(world, block, &self.pos, state_id, &self.block_entity)
    }

    fn try_create_at(pos: BlockPos, world: &World) -> Option<Self> {
        let block_entity = world.get_block_entity(&pos)?;
        Self::try_create(block_entity, world)
    }
}

pub type BlockPredicate = fn(&'static Block) -> bool;
pub type TargetInteractionCallback =
    fn(&dyn Mob, ContainerInteractionState, &TransportItemTarget, i32);
pub type TravellingCallback = fn(&dyn Mob);
pub type QueuePredicate = fn(&TransportItemTarget) -> bool;

pub struct TransportItemsBetweenContainers {
    speed_modifier: f32,
    horizontal_search_distance: i32,
    vertical_search_distance: i32,
    source_block_type: BlockPredicate,
    destination_block_type: BlockPredicate,
    on_target_interaction: TargetInteractionCallback,
    on_start_travelling: TravellingCallback,
    should_queue_for_target: QueuePredicate,
    status: Status,
    target: Option<TransportItemTarget>,
    state: TransportItemState,
    interaction_state: Option<ContainerInteractionState>,
    ticks_since_reaching_target: i32,
}

impl TransportItemsBetweenContainers {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        speed_modifier: f32,
        source_block_type: BlockPredicate,
        destination_block_type: BlockPredicate,
        horizontal_search_distance: i32,
        vertical_search_distance: i32,
        on_target_interaction: TargetInteractionCallback,
        on_start_travelling: TravellingCallback,
        should_queue_for_target: QueuePredicate,
    ) -> Self {
        Self {
            speed_modifier,
            horizontal_search_distance,
            vertical_search_distance,
            source_block_type,
            destination_block_type,
            on_target_interaction,
            on_start_travelling,
            should_queue_for_target,
            status: Status::Stopped,
            target: None,
            state: TransportItemState::Travelling,
            interaction_state: None,
            ticks_since_reaching_target: 0,
        }
    }

    fn can_still_use(tick: &BrainTick<'_>) -> bool {
        !tick
            .state
            .has_memory_value(types::TRANSPORT_ITEMS_COOLDOWN_TICKS.id())
            && !tick.state.has_memory_value(types::IS_PANICKING.id())
            && !tick.mob.get_entity().is_leashed()
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let updated_invalid_target = self.update_invalid_target(tick);
        if self.target.is_none() {
            self.do_stop(tick);
        } else if !updated_invalid_target {
            if self.state == TransportItemState::Queuing {
                self.on_queuing_for_target(tick);
            }
            if self.state == TransportItemState::Travelling {
                self.on_travel_to_target(tick);
            }
            if self.state == TransportItemState::Interacting {
                self.on_reached_target(tick);
            }
        }
    }

    fn update_invalid_target(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if self.has_valid_target(tick) {
            return false;
        }
        self.stop_targeting_current_target(tick);
        if let Some(target) = self.get_transport_target(tick) {
            let target_pos = target.pos;
            self.target = Some(target);
            self.on_start_travelling(tick);
            self.set_visited_block_pos(tick, target_pos);
        } else {
            self.enter_cooldown_after_no_matching_target_found(tick);
        }
        true
    }

    fn on_queuing_for_target(&mut self, tick: &mut BrainTick<'_>) {
        let busy = self
            .target
            .as_ref()
            .is_some_and(|target| self.is_another_mob_interacting_with_target(target, tick.world));
        if !busy {
            self.state = TransportItemState::Travelling;
            self.walk_towards_target(tick);
        }
    }

    fn on_travel_to_target(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = self.target.as_ref() else {
            return;
        };
        let center = Self::get_center_pos(tick.mob);
        if Self::is_within_target_distance(
            CLOSE_ENOUGH_TO_START_QUEUING_DISTANCE,
            target,
            tick.world,
            tick.mob,
            center,
        ) && self.is_another_mob_interacting_with_target(target, tick.world)
        {
            Self::stop_in_place(tick.mob);
            self.state = TransportItemState::Queuing;
        } else if Self::is_within_target_distance(
            Self::get_interaction_range(tick.mob),
            target,
            tick.world,
            tick.mob,
            center,
        ) {
            let Some(container) = target.container(tick.world) else {
                self.stop_targeting_current_target(tick);
                return;
            };
            self.interaction_state = Some(Self::choose_interaction_state(tick.mob, &container));
            self.state = TransportItemState::Interacting;
        } else {
            self.walk_towards_target(tick);
        }
    }

    fn on_reached_target(&mut self, tick: &mut BrainTick<'_>) {
        let still_close = self.target.as_ref().is_some_and(|target| {
            Self::is_within_target_distance(
                CLOSE_ENOUGH_TO_CONTINUE_INTERACTING_WITH_TARGET,
                target,
                tick.world,
                tick.mob,
                Self::get_center_pos(tick.mob),
            )
        });
        if !still_close {
            self.on_start_travelling(tick);
            return;
        }

        self.ticks_since_reaching_target += 1;
        if let Some(target) = self.target.as_ref() {
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(BlockPosTracker::new(target.pos)),
            );
            Self::stop_in_place(tick.mob);
            if let Some(interaction_state) = self.interaction_state {
                (self.on_target_interaction)(
                    tick.mob,
                    interaction_state,
                    target,
                    self.ticks_since_reaching_target,
                );
            }
        }

        if self.ticks_since_reaching_target >= TARGET_INTERACTION_TIME {
            let container = self
                .target
                .as_ref()
                .and_then(|target| target.container(tick.world));
            if let Some(container) = container {
                if Self::is_picking_up_items(tick.mob) {
                    if Self::matches_getting_items_requirement(&container) {
                        self.pick_up_items(tick, &container);
                    } else {
                        self.stop_targeting_current_target(tick);
                    }
                } else if Self::matches_leaving_items_requirement(tick.mob, &container) {
                    self.put_down_item(tick, &container);
                } else {
                    self.stop_targeting_current_target(tick);
                }
            }
            self.on_start_travelling(tick);
        }
    }

    fn on_start_travelling(&mut self, tick: &BrainTick<'_>) {
        (self.on_start_travelling)(tick.mob);
        self.state = TransportItemState::Travelling;
        self.interaction_state = None;
        self.ticks_since_reaching_target = 0;
    }

    fn walk_towards_target(&self, tick: &mut BrainTick<'_>) {
        if let Some(target) = self.target.as_ref() {
            utils::set_walk_and_look_target_memories_to_block(
                tick.state,
                target.pos,
                self.speed_modifier,
                0,
            );
        }
    }

    fn stop_in_place(mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        mob_entity.steered().navigator().stop();
        mob_entity
            .living_entity
            .movement_input
            .store(Vector3::default());
        mob_entity.set_speed(0.0);
        let entity = &mob_entity.living_entity.entity;
        let velocity = entity.velocity.load();
        entity.velocity.store(Vector3::new(0.0, velocity.y, 0.0));
    }

    fn choose_interaction_state(
        mob: &dyn Mob,
        container: &Arc<dyn Inventory>,
    ) -> ContainerInteractionState {
        if Self::is_picking_up_items(mob) {
            if Self::matches_getting_items_requirement(container) {
                ContainerInteractionState::PickupItem
            } else {
                ContainerInteractionState::PickupNoItem
            }
        } else if Self::matches_leaving_items_requirement(mob, container) {
            ContainerInteractionState::PlaceItem
        } else {
            ContainerInteractionState::PlaceNoItem
        }
    }

    fn get_transport_target(&self, tick: &BrainTick<'_>) -> Option<TransportItemTarget> {
        let mob = tick.mob;
        let world = tick.world;
        let search_area = self.get_target_search_area(mob);
        let visited = tick
            .state
            .get(types::VISITED_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        let unreachable = tick
            .state
            .get(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();

        let block_pos = mob.get_entity().block_pos.load();
        let mob_pos = mob.get_entity().pos.load();
        let center_chunk_x = block_pos.0.x.div_euclid(16);
        let center_chunk_z = block_pos.0.z.div_euclid(16);
        let chunk_radius = self.get_horizontal_search_distance(mob).div_euclid(16) + 1;

        let mut best: Option<TransportItemTarget> = None;
        let mut closest_dist_sq = f64::from(f32::MAX);

        for cx in (center_chunk_x - chunk_radius)..=(center_chunk_x + chunk_radius) {
            for cz in (center_chunk_z - chunk_radius)..=(center_chunk_z + chunk_radius) {
                // Released before the candidates are looked at: checking one can reach plugins.
                let chests: Vec<Arc<dyn BlockEntity>> = {
                    let Some(chunk_bes) = world.block_entities.get(&Vector2::new(cx, cz)) else {
                        continue;
                    };
                    chunk_bes
                        .values()
                        .filter(|block_entity| is_chest_block_entity(block_entity.as_ref()))
                        .cloned()
                        .collect()
                };
                for block_entity in &chests {
                    let dist_sq = block_entity
                        .get_position()
                        .to_centered_f64()
                        .squared_distance_to_vec(&mob_pos);
                    if dist_sq < closest_dist_sq
                        && let Some(candidate) = self.is_target_valid_to_pick(
                            mob,
                            world,
                            Arc::clone(block_entity),
                            &visited,
                            &unreachable,
                            &search_area,
                        )
                    {
                        best = Some(candidate);
                        closest_dist_sq = dist_sq;
                    }
                }
            }
        }

        best
    }

    fn is_target_valid_to_pick(
        &self,
        mob: &dyn Mob,
        world: &World,
        block_entity: Arc<dyn BlockEntity>,
        visited: &FxHashSet<GlobalPos>,
        unreachable: &FxHashSet<GlobalPos>,
        search_area: &BoundingBox,
    ) -> Option<TransportItemTarget> {
        let pos = block_entity.get_position();
        if !search_area.contains(Vector3::new(
            f64::from(pos.0.x),
            f64::from(pos.0.y),
            f64::from(pos.0.z),
        )) {
            return None;
        }
        let target = TransportItemTarget::try_create(block_entity, world)?;
        let is_valid = self.is_wanted_block(mob, target.block)
            && !Self::is_position_already_visited(visited, unreachable, &target, world)
            && target.block_entity.get_lock().is_none();
        is_valid.then_some(target)
    }

    fn has_valid_target(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(target) = self.target.as_ref() else {
            return false;
        };
        let valid_type = self.is_wanted_block(tick.mob, target.block)
            && Self::target_has_not_changed(tick.world, target);
        if valid_type && !Self::is_target_blocked(tick.world, target) {
            if self.state != TransportItemState::Travelling {
                return true;
            }
            if Self::has_valid_travelling_path(tick.world, target, tick.mob) {
                return true;
            }
            let target_pos = target.pos;
            self.mark_visited_block_pos_as_unreachable(tick, target_pos);
        }
        false
    }

    fn has_valid_travelling_path(
        world: &World,
        target: &TransportItemTarget,
        mob: &dyn Mob,
    ) -> bool {
        let mob_entity = mob.get_mob_entity();
        let end_node_pos = {
            let steered = mob_entity.steered();
            let mut nav = steered.navigator();
            let existing = nav
                .get_path()
                .and_then(|path| path.get_end_node().map(|n| n.pos));
            existing.or_else(|| {
                nav.create_path(&mob_entity.living_entity, target.pos.0.to_f64(), 0)
                    .and_then(|p| p.get_end_node().map(|n| n.pos))
            })
        };
        let has_path = end_node_pos.is_some();
        let bottom_center = end_node_pos.map_or_else(
            || mob.get_entity().pos.load(),
            |pos| {
                Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y),
                    f64::from(pos.0.z) + 0.5,
                )
            },
        );
        let pos_from = Self::set_middle_y_position(mob, bottom_center);
        let can_reach = Self::is_within_target_distance(
            Self::get_interaction_range(mob),
            target,
            world,
            mob,
            pos_from,
        );
        let has_not_yet_created_path = !has_path && !can_reach;
        has_not_yet_created_path
            || (can_reach && Self::can_see_any_target_side(target, world, pos_from))
    }

    fn get_center_pos(mob: &dyn Mob) -> Vector3<f64> {
        Self::set_middle_y_position(mob, mob.get_entity().pos.load())
    }

    fn set_middle_y_position(mob: &dyn Mob, pos: Vector3<f64>) -> Vector3<f64> {
        let bb = mob.get_entity().bounding_box.load();
        pos + Vector3::new(0.0, (bb.max.y - bb.min.y) * 0.5, 0.0)
    }

    fn is_target_blocked(world: &World, target: &TransportItemTarget) -> bool {
        if chests::is_chest_blocked(world, &target.pos) {
            return true;
        }
        let props = ChestLikeProperties::from_state_id(target.state_id);
        chests::connected_half(world, target.block, &target.pos, props)
            .is_some_and(|other| chests::is_chest_blocked(world, &other))
    }

    fn target_has_not_changed(world: &World, target: &TransportItemTarget) -> bool {
        world
            .get_block_entity(&target.pos)
            .is_some_and(|current| Arc::ptr_eq(&current, &target.block_entity))
    }

    fn connected_pos(target: &TransportItemTarget, world: &World) -> Option<BlockPos> {
        let props = ChestLikeProperties::from_state_id(target.state_id);
        if props.r#type == ChestType::Single {
            return None;
        }
        chests::connected_half(world, target.block, &target.pos, props)
    }

    fn get_target_search_area(&self, mob: &dyn Mob) -> BoundingBox {
        let h = f64::from(self.get_horizontal_search_distance(mob));
        let v = f64::from(self.get_vertical_search_distance(mob));
        BoundingBox::from_block(&mob.get_entity().block_pos.load()).expand(h, v, h)
    }

    fn get_horizontal_search_distance(&self, mob: &dyn Mob) -> i32 {
        if mob.is_passenger() {
            PASSENGER_MOB_TARGET_SEARCH_DISTANCE
        } else {
            self.horizontal_search_distance
        }
    }

    fn get_vertical_search_distance(&self, mob: &dyn Mob) -> i32 {
        if mob.is_passenger() {
            PASSENGER_MOB_TARGET_SEARCH_DISTANCE
        } else {
            self.vertical_search_distance
        }
    }

    fn is_position_already_visited(
        visited: &FxHashSet<GlobalPos>,
        unreachable: &FxHashSet<GlobalPos>,
        target: &TransportItemTarget,
        world: &World,
    ) -> bool {
        let Some(dimension) = Dimension::from_name(world.dimension.minecraft_name) else {
            return false;
        };
        let gp = GlobalPos::new(dimension, target.pos);
        if visited.contains(&gp) || unreachable.contains(&gp) {
            return true;
        }
        if let Some(other_pos) = Self::connected_pos(target, world) {
            let other_gp = GlobalPos::new(dimension, other_pos);
            if visited.contains(&other_gp) || unreachable.contains(&other_gp) {
                return true;
            }
        }
        false
    }

    fn has_finished_path(mob: &dyn Mob) -> bool {
        let steered = mob.get_mob_entity().steered();
        let nav = steered.navigator();
        nav.get_path()
            .is_some_and(crate::entity::ai::pathfinder::path::Path::is_done)
    }

    fn set_visited_block_pos(&mut self, tick: &mut BrainTick<'_>, target: BlockPos) {
        let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
            return;
        };
        let mut visited = tick
            .state
            .get(types::VISITED_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        visited.insert(GlobalPos::new(dimension, target));
        if visited.len() > MAX_VISITED_POSITIONS {
            self.enter_cooldown_after_no_matching_target_found(tick);
        } else {
            tick.state.set_with_expiry(
                types::VISITED_BLOCK_POSITIONS,
                visited,
                VISITED_POSITIONS_MEMORY_TIME,
            );
        }
    }

    fn mark_visited_block_pos_as_unreachable(
        &mut self,
        tick: &mut BrainTick<'_>,
        target: BlockPos,
    ) {
        let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
            return;
        };
        let gp = GlobalPos::new(dimension, target);
        let mut visited = tick
            .state
            .get(types::VISITED_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        visited.remove(&gp);
        let mut unreachable = tick
            .state
            .get(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS)
            .cloned()
            .unwrap_or_default();
        unreachable.insert(gp);
        if unreachable.len() > MAX_UNREACHABLE_POSITIONS {
            self.enter_cooldown_after_no_matching_target_found(tick);
        } else {
            tick.state.set_with_expiry(
                types::VISITED_BLOCK_POSITIONS,
                visited,
                VISITED_POSITIONS_MEMORY_TIME,
            );
            tick.state.set_with_expiry(
                types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS,
                unreachable,
                VISITED_POSITIONS_MEMORY_TIME,
            );
        }
    }

    fn is_wanted_block(&self, mob: &dyn Mob, block: &'static Block) -> bool {
        if Self::is_picking_up_items(mob) {
            (self.source_block_type)(block)
        } else {
            (self.destination_block_type)(block)
        }
    }

    fn get_interaction_range(mob: &dyn Mob) -> f64 {
        if Self::has_finished_path(mob) {
            CLOSE_ENOUGH_TO_START_INTERACTING_WITH_TARGET_PATH_END_DISTANCE
        } else {
            CLOSE_ENOUGH_TO_START_INTERACTING_WITH_TARGET_DISTANCE
        }
    }

    fn is_within_target_distance(
        distance: f64,
        target: &TransportItemTarget,
        world: &World,
        mob: &dyn Mob,
        from_pos: Vector3<f64>,
    ) -> bool {
        let bb = mob.get_entity().bounding_box.load();
        let half_x = (bb.max.x - bb.min.x) * 0.5;
        let half_y = (bb.max.y - bb.min.y) * 0.5;
        let half_z = (bb.max.z - bb.min.z) * 0.5;
        let moved_bb = BoundingBox::new(
            Vector3::new(
                from_pos.x - half_x,
                from_pos.y - half_y,
                from_pos.z - half_z,
            ),
            Vector3::new(
                from_pos.x + half_x,
                from_pos.y + half_y,
                from_pos.z + half_z,
            ),
        );
        let state = world.get_block_state(&target.pos);
        let target_bb = state
            .get_block_collision_shapes()
            .reduce(|acc, shape| {
                BoundingBox::new(
                    Vector3::new(
                        acc.min.x.min(shape.min.x),
                        acc.min.y.min(shape.min.y),
                        acc.min.z.min(shape.min.z),
                    ),
                    Vector3::new(
                        acc.max.x.max(shape.max.x),
                        acc.max.y.max(shape.max.y),
                        acc.max.z.max(shape.max.z),
                    ),
                )
            })
            .unwrap_or_else(|| BoundingBox::new(Vector3::default(), Vector3::new(1.0, 1.0, 1.0)))
            .expand(distance, 0.5, distance)
            .shift(Vector3::new(
                f64::from(target.pos.0.x),
                f64::from(target.pos.0.y),
                f64::from(target.pos.0.z),
            ));
        target_bb.intersects(&moved_bb)
    }

    fn can_see_any_target_side(
        target: &TransportItemTarget,
        world: &World,
        eye_position: Vector3<f64>,
    ) -> bool {
        let center = target.pos.to_centered_f64();
        SIDE_OFFSETS.iter().any(|&(dx, dy, dz)| {
            let hit_target = Vector3::new(center.x + dx, center.y + dy, center.z + dz);
            let hit = world.clip(&ClipContext::new(
                eye_position,
                hit_target,
                ClipBlock::Collider,
                ClipFluid::None,
            ));
            !hit.is_miss && hit.block_pos == target.pos
        })
    }

    fn is_another_mob_interacting_with_target(
        &self,
        target: &TransportItemTarget,
        world: &World,
    ) -> bool {
        if (self.should_queue_for_target)(target) {
            return true;
        }
        Self::connected_pos(target, world)
            .and_then(|pos| TransportItemTarget::try_create_at(pos, world))
            .is_some_and(|other| (self.should_queue_for_target)(&other))
    }

    fn is_picking_up_items(mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .is_empty()
    }

    fn matches_getting_items_requirement(container: &Arc<dyn Inventory>) -> bool {
        !container.is_empty()
    }

    fn matches_leaving_items_requirement(mob: &dyn Mob, container: &Arc<dyn Inventory>) -> bool {
        if container.is_empty() {
            return true;
        }
        let main_hand = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        for slot in 0..container.size() {
            let stack = container.get_stack(slot);
            if !stack.is_empty() && stack.item.id == main_hand.item.id {
                return true;
            }
        }
        false
    }

    fn pick_up_items(&mut self, tick: &mut BrainTick<'_>, container: &Arc<dyn Inventory>) {
        let picked = Self::pickup_item_from_container(container);
        tick.mob
            .get_mob_entity()
            .set_item_slot(&EquipmentSlot::MAIN_HAND, picked);
        tick.mob
            .get_mob_entity()
            .set_guaranteed_drop(&EquipmentSlot::MAIN_HAND);
        container.mark_dirty();
        self.clear_memories_after_matching_target_found(tick);
    }

    fn put_down_item(&mut self, tick: &mut BrainTick<'_>, container: &Arc<dyn Inventory>) {
        let remainder = Self::add_items_to_container(tick.mob, container);
        container.mark_dirty();
        let is_empty = remainder.is_empty();
        tick.mob
            .get_mob_entity()
            .set_item_slot(&EquipmentSlot::MAIN_HAND, remainder);
        if is_empty {
            self.clear_memories_after_matching_target_found(tick);
        } else {
            self.stop_targeting_current_target(tick);
        }
    }

    fn pickup_item_from_container(container: &Arc<dyn Inventory>) -> ItemStack {
        for slot in 0..container.size() {
            let stack = container.get_stack(slot);
            if !stack.is_empty() {
                let count = stack.item_count.min(TRANSPORTED_ITEM_MAX_STACK_SIZE);
                return container.remove_stack_specific(slot, count);
            }
        }
        ItemStack::EMPTY.clone()
    }

    fn add_items_to_container(mob: &dyn Mob, container: &Arc<dyn Inventory>) -> ItemStack {
        let mut stack = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        if stack.is_empty() {
            return ItemStack::EMPTY.clone();
        }
        for slot in 0..container.size() {
            let mut emptied = false;
            container.update_stack(slot, &mut |slot_stack| {
                if slot_stack.is_empty() {
                    *slot_stack = std::mem::replace(&mut stack, ItemStack::EMPTY.clone());
                    emptied = true;
                    return true;
                }
                let max_size = slot_stack.get_max_stack_size();
                if !slot_stack.are_items_and_components_equal(&stack)
                    || slot_stack.item_count >= max_size
                {
                    return false;
                }
                let to_add = (max_size - slot_stack.item_count).min(stack.item_count);
                slot_stack.item_count += to_add;
                stack.decrement(to_add);
                emptied = stack.is_empty();
                true
            });
            if emptied {
                return ItemStack::EMPTY.clone();
            }
        }
        stack
    }

    fn stop_targeting_current_target(&mut self, tick: &mut BrainTick<'_>) {
        self.ticks_since_reaching_target = 0;
        self.target = None;
        Self::stop_in_place(tick.mob);
        tick.state.erase(types::WALK_TARGET.id());
    }

    fn clear_memories_after_matching_target_found(&mut self, tick: &mut BrainTick<'_>) {
        self.stop_targeting_current_target(tick);
        tick.state.erase(types::VISITED_BLOCK_POSITIONS.id());
        tick.state
            .erase(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS.id());
    }

    fn enter_cooldown_after_no_matching_target_found(&mut self, tick: &mut BrainTick<'_>) {
        self.stop_targeting_current_target(tick);
        tick.state
            .set(types::TRANSPORT_ITEMS_COOLDOWN_TICKS, IDLE_COOLDOWN);
        tick.state.erase(types::VISITED_BLOCK_POSITIONS.id());
        tick.state
            .erase(types::UNREACHABLE_TRANSPORT_BLOCK_POSITIONS.id());
    }
}

impl BehaviorControl for TransportItemsBetweenContainers {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &REQUIRED_MEMORIES
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !tick.state.check_all(&ENTRY_CONDITIONS) || tick.mob.get_entity().is_leashed() {
            return false;
        }
        self.status = Status::Running;
        tick.mob
            .get_mob_entity()
            .steered()
            .navigator()
            .set_can_path_to_targets_below_surface(true);
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        if Self::can_still_use(tick) {
            self.tick(tick);
        } else {
            self.do_stop(tick);
        }
    }

    fn do_stop(&mut self, tick: &mut BrainTick<'_>) {
        self.status = Status::Stopped;
        self.on_start_travelling(tick);
        tick.mob
            .get_mob_entity()
            .steered()
            .navigator()
            .set_can_path_to_targets_below_surface(false);
    }

    fn debug_string(&self) -> String {
        "TransportItemsBetweenContainers".to_string()
    }
}

fn is_chest_block_entity(block_entity: &dyn BlockEntity) -> bool {
    block_entity.as_any().is::<ChestBlockEntity>()
        || block_entity.as_any().is::<TrappedChestBlockEntity>()
}
