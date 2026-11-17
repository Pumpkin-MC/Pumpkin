use std::collections::VecDeque;
use std::sync::Arc;

use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::{AcaciaShelfLikeProperties, HorizontalFacing, SideChainPart};
use pumpkin_data::data_component_impl::{EquipmentSlot, UseEffectsImpl};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_inventory::Inventory;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::shelf::ShelfBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnStateReplacedArgs, PathComputationType, PlacedArgs, UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::world::World;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct ShelfBlock;

impl ShelfBlock {
    const ROWS: usize = 1;
    const COLUMNS: usize = 3;
    const MAX_CHAIN_LENGTH: usize = 3;

    fn swap_single_item(
        args: &UseWithItemArgs<'_>,
        shelf: &ShelfBlockEntity,
        hit_slot: usize,
    ) -> BlockActionResult {
        let placed_stack = args.item_stack.clone();
        let removed_item = shelf.swap_item_no_update(hit_slot, placed_stack.clone());
        let item_removed = !removed_item.is_empty();
        let new_inventory_item =
            if args.player.gamemode.load() == GameMode::Creative && removed_item.is_empty() {
                placed_stack.clone()
            } else {
                removed_item
            };

        let selected_slot = args.player.inventory().get_selected_slot() as usize;
        args.player
            .inventory()
            .set_held_item(new_inventory_item.clone());
        args.player
            .sync_hand_slot(selected_slot, new_inventory_item.clone());

        let event = if new_inventory_item
            .get_data_component::<UseEffectsImpl>()
            .is_some_and(|effects| !effects.interact_vibrations)
        {
            None
        } else {
            Some(GameEvent::ItemInteractFinish)
        };
        shelf.mark_dirty_with_event(event);

        if item_removed {
            let sound = if placed_stack.is_empty() {
                Sound::BlockShelfTakeItem
            } else {
                Sound::BlockShelfSingleSwap
            };
            args.world
                .play_block_sound(sound, SoundCategory::Blocks, *args.position);
        } else {
            if placed_stack.is_empty() {
                return BlockActionResult::Pass;
            }
            args.world.play_block_sound(
                Sound::BlockShelfPlaceItem,
                SoundCategory::Blocks,
                *args.position,
            );
        }

        BlockActionResult::SuccessServer
    }

    fn swap_hotbar(args: &UseWithItemArgs<'_>) -> BlockActionResult {
        let connected_blocks = Self::get_all_blocks_connected_to(args.world, args.position);
        if connected_blocks.is_empty() {
            return BlockActionResult::Consume;
        }

        let inventory = args.player.inventory();
        let selected_slot = inventory.get_selected_slot() as usize;
        let mut any_swapped = false;

        for (shelf_part_index, shelf_pos) in connected_blocks.iter().enumerate() {
            let Some(block_entity) = args.world.get_block_entity(shelf_pos) else {
                continue;
            };
            let Some(shelf_part) = block_entity.as_any().downcast_ref::<ShelfBlockEntity>() else {
                continue;
            };

            for slot in 0..ShelfBlockEntity::INVENTORY_SIZE {
                let Some(inventory_slot) =
                    shelf_hotbar_slot(connected_blocks.len(), shelf_part_index, slot)
                else {
                    continue;
                };
                let placed_inventory_item = inventory.remove_stack(inventory_slot);
                let removed_shelf_item =
                    shelf_part.swap_item_no_update(slot, placed_inventory_item.clone());
                if !placed_inventory_item.is_empty() || !removed_shelf_item.is_empty() {
                    inventory.set_stack(inventory_slot, removed_shelf_item.clone());
                    args.player
                        .try_send_slot_set_packet(&CSetPlayerInventory::new(
                            (inventory_slot as i32).into(),
                            &ItemStackSerializer::from(removed_shelf_item.clone()),
                        ));
                    if inventory_slot == selected_slot {
                        args.player.living_entity.send_equipment_changes(&[(
                            EquipmentSlot::MAIN_HAND,
                            removed_shelf_item,
                        )]);
                    }
                    any_swapped = true;
                }
            }

            shelf_part.mark_dirty_with_event(Some(GameEvent::EntityInteract));
        }

        args.player.sync_inventory_to_client();
        if !any_swapped {
            return BlockActionResult::Consume;
        }

        args.world.play_block_sound(
            Sound::BlockShelfMultiSwap,
            SoundCategory::Blocks,
            *args.position,
        );
        BlockActionResult::SuccessServer
    }

    fn is_connectable(block: &Block, state_id: BlockStateId) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_WOODEN_SHELVES)
            && AcaciaShelfLikeProperties::from_state_id(state_id).powered
    }

    fn connectable_part_at(
        world: &World,
        pos: &BlockPos,
        facing: HorizontalFacing,
    ) -> Option<SideChainPart> {
        let (block, state_id) = world.get_block_and_state_id(pos);
        if !block.has_tag(&tag::Block::MINECRAFT_WOODEN_SHELVES) {
            return None;
        }
        let props = AcaciaShelfLikeProperties::from_state_id(state_id);
        (props.powered && props.facing == facing).then_some(props.r#side_chain)
    }

    fn set_part(world: &Arc<World>, pos: &BlockPos, new_part: SideChainPart) {
        let (block, state_id) = world.get_block_and_state_id(pos);
        if !block.has_tag(&tag::Block::MINECRAFT_WOODEN_SHELVES) {
            return;
        }
        let mut props = AcaciaShelfLikeProperties::from_state_id(state_id);
        if props.r#side_chain != new_part {
            props.r#side_chain = new_part;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        }
    }

    fn step_left(center: &BlockPos, facing: HorizontalFacing, steps: i32) -> BlockPos {
        let offset = facing.to_block_direction().rotate_clockwise().to_offset();
        BlockPos::new(
            center.0.x + offset.x * steps,
            center.0.y + offset.y * steps,
            center.0.z + offset.z * steps,
        )
    }

    fn step_right(center: &BlockPos, facing: HorizontalFacing, steps: i32) -> BlockPos {
        let offset = facing
            .to_block_direction()
            .rotate_counter_clockwise()
            .to_offset();
        BlockPos::new(
            center.0.x + offset.x * steps,
            center.0.y + offset.y * steps,
            center.0.z + offset.z * steps,
        )
    }

    /// Vanilla `SideChainPartBlock.getAllBlocksConnectedTo`.
    fn get_all_blocks_connected_to(world: &World, pos: &BlockPos) -> Vec<BlockPos> {
        let (block, state_id) = world.get_block_and_state_id(pos);
        if !Self::is_connectable(block, state_id) {
            return Vec::new();
        }
        let facing = AcaciaShelfLikeProperties::from_state_id(state_id).facing;
        let mut results = VecDeque::with_capacity(Self::MAX_CHAIN_LENGTH);
        results.push_back(*pos);

        for steps in 1..(Self::MAX_CHAIN_LENGTH as i32) {
            let neighbor_pos = Self::step_left(pos, facing, steps);
            let Some(part) = Self::connectable_part_at(world, &neighbor_pos, facing) else {
                break;
            };
            if side_chain_connects_towards(part, SideChainPart::Left) {
                results.push_front(neighbor_pos);
            }
            if side_chain_is_end(part) {
                break;
            }
        }

        for steps in 1..(Self::MAX_CHAIN_LENGTH as i32) {
            let neighbor_pos = Self::step_right(pos, facing, steps);
            let Some(part) = Self::connectable_part_at(world, &neighbor_pos, facing) else {
                break;
            };
            if side_chain_connects_towards(part, SideChainPart::Right) {
                results.push_back(neighbor_pos);
            }
            if side_chain_is_end(part) {
                break;
            }
        }

        Vec::from(results)
    }

    /// Vanilla `SideChainPartBlock.updateNeighborsAfterPoweringDown`.
    fn update_neighbors_after_powering_down(
        world: &Arc<World>,
        pos: &BlockPos,
        facing: HorizontalFacing,
    ) {
        let left_pos = Self::step_left(pos, facing, 1);
        if let Some(part) = Self::connectable_part_at(world, &left_pos, facing) {
            Self::set_part(world, &left_pos, side_chain_disconnected_from_right(part));
        }
        let right_pos = Self::step_right(pos, facing, 1);
        if let Some(part) = Self::connectable_part_at(world, &right_pos, facing) {
            Self::set_part(world, &right_pos, side_chain_disconnected_from_left(part));
        }
    }

    /// Vanilla `SideChainPartBlock.updateSelfAndNeighborsOnPoweringUp`.
    fn update_self_and_neighbors_on_powering_up(
        world: &Arc<World>,
        pos: &BlockPos,
        state_id: BlockStateId,
        old_state_id: BlockStateId,
    ) {
        let block = Block::from_state_id(state_id);
        if !Self::is_connectable(block, state_id) {
            return;
        }
        let props = AcaciaShelfLikeProperties::from_state_id(state_id);
        let old_block = Block::from_state_id(old_state_id);
        let has_been_connected_before = Self::is_connectable(old_block, old_state_id)
            && side_chain_is_connected(
                AcaciaShelfLikeProperties::from_state_id(old_state_id).r#side_chain,
            );
        if side_chain_is_connected(props.r#side_chain) || has_been_connected_before {
            return;
        }

        let facing = props.facing;
        let left_pos = Self::step_left(pos, facing, 1);
        let right_pos = Self::step_right(pos, facing, 1);
        let left_part = Self::connectable_part_at(world, &left_pos, facing);
        let right_part = Self::connectable_part_at(world, &right_pos, facing);

        let existing_left = if left_part.is_some() {
            Self::get_all_blocks_connected_to(world, &left_pos).len()
        } else {
            0
        };
        let existing_right = if right_part.is_some() {
            Self::get_all_blocks_connected_to(world, &right_pos).len()
        } else {
            0
        };

        let mut new_part_for_self = SideChainPart::Unconnected;
        let mut current_chain_length = 1usize;

        if existing_left > 0 && current_chain_length + existing_left <= Self::MAX_CHAIN_LENGTH {
            new_part_for_self = side_chain_connected_to_left(new_part_for_self);
            if let Some(lp) = left_part {
                Self::set_part(world, &left_pos, side_chain_connected_to_right(lp));
            }
            current_chain_length += existing_left;
        }

        if existing_right > 0 && current_chain_length + existing_right <= Self::MAX_CHAIN_LENGTH {
            new_part_for_self = side_chain_connected_to_right(new_part_for_self);
            if let Some(rp) = right_part {
                Self::set_part(world, &right_pos, side_chain_connected_to_left(rp));
            }
        }

        Self::set_part(world, pos, new_part_for_self);
    }
}

impl BlockBehaviour for ShelfBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = AcaciaShelfLikeProperties::default(args.block);

        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();
        properties.powered = block_receives_redstone_power(args.world, args.position);
        properties.waterlogged = args.replacing.water_source();

        properties.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = AcaciaShelfLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = ShelfBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));

        let props = AcaciaShelfLikeProperties::from_state_id(args.state_id);
        if props.powered {
            Self::update_self_and_neighbors_on_powering_up(
                args.world,
                args.position,
                args.state_id,
                args.old_state_id,
            );
        } else {
            Self::update_neighbors_after_powering_down(args.world, args.position, props.facing);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world
            .update_neighbour_for_output_signal(args.position, args.block);
        let props = AcaciaShelfLikeProperties::from_state_id(args.old_state_id);
        Self::update_neighbors_after_powering_down(args.world, args.position, props.facing);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = AcaciaShelfLikeProperties::from_state_id(state_id);
        let signal = block_receives_redstone_power(args.world, args.position);
        if props.powered == signal {
            return;
        }

        props.powered = signal;
        if !signal {
            props.r#side_chain = SideChainPart::Unconnected;
        }
        let new_state_id = props.to_state_id(args.block);
        args.world
            .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);

        if signal {
            let current_state_id = args.world.get_block_state_id(args.position);
            Self::update_self_and_neighbors_on_powering_up(
                args.world,
                args.position,
                current_state_id,
                state_id,
            );
        } else {
            Self::update_neighbors_after_powering_down(args.world, args.position, props.facing);
        }

        let (sound, event) = if signal {
            (Sound::BlockShelfActivate, GameEvent::BlockActivate)
        } else {
            (Sound::BlockShelfDeactivate, GameEvent::BlockDeactivate)
        };
        args.world
            .play_block_sound(sound, SoundCategory::Blocks, *args.position);
        args.world
            .emit_game_event(event.name(), args.position.to_centered_f64());
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if *args.equipment_slot == EquipmentSlot::OFF_HAND {
            return BlockActionResult::Pass;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        let Some(shelf) = block_entity.as_any().downcast_ref::<ShelfBlockEntity>() else {
            return BlockActionResult::Pass;
        };

        let state_id = args.world.get_block_state_id(args.position);
        let props = AcaciaShelfLikeProperties::from_state_id(state_id);
        let Some(hit_slot) = args
            .hit
            .get_hit_slot(props.facing, Self::ROWS, Self::COLUMNS)
        else {
            return BlockActionResult::Pass;
        };

        if props.powered {
            Self::swap_hotbar(&args)
        } else {
            Self::swap_single_item(&args, shelf, hit_slot)
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla reads a shelf only through its back face.
        let props = AcaciaShelfLikeProperties::from_state_id(args.state.id);
        if args.direction != props.facing.to_block_direction().opposite() {
            return Some(0);
        }

        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(shelf) = block_entity.as_any().downcast_ref::<ShelfBlockEntity>()
        {
            let b0 = u8::from(!shelf.get_stack(0).is_empty());
            let b1 = u8::from(!shelf.get_stack(1).is_empty());
            let b2 = u8::from(!shelf.get_stack(2).is_empty());
            Some(b0 | (b1 << 1) | (b2 << 2))
        } else {
            None
        }
    }

    fn is_pathfindable(&self, state: &BlockState, computation_type: PathComputationType) -> bool {
        computation_type == PathComputationType::Water && state.is_waterlogged()
    }
}

const fn shelf_hotbar_slot(
    connected_len: usize,
    shelf_part_index: usize,
    slot: usize,
) -> Option<usize> {
    let raw = 9isize
        - ((connected_len.saturating_sub(shelf_part_index)) * ShelfBlockEntity::INVENTORY_SIZE)
            as isize
        + slot as isize;
    if raw >= 0 && raw < 9 {
        Some(raw as usize)
    } else {
        None
    }
}

const fn side_chain_is_connected(part: SideChainPart) -> bool {
    !matches!(part, SideChainPart::Unconnected)
}

const fn side_chain_connects_towards(part: SideChainPart, end_part: SideChainPart) -> bool {
    matches!(part, SideChainPart::Center)
        || matches!(
            (part, end_part),
            (SideChainPart::Left, SideChainPart::Left)
                | (SideChainPart::Right, SideChainPart::Right)
        )
}

const fn side_chain_is_end(part: SideChainPart) -> bool {
    !matches!(part, SideChainPart::Center)
}

const fn side_chain_connected_to_right(part: SideChainPart) -> SideChainPart {
    match part {
        SideChainPart::Unconnected | SideChainPart::Left => SideChainPart::Left,
        SideChainPart::Right | SideChainPart::Center => SideChainPart::Center,
    }
}

const fn side_chain_connected_to_left(part: SideChainPart) -> SideChainPart {
    match part {
        SideChainPart::Unconnected | SideChainPart::Right => SideChainPart::Right,
        SideChainPart::Center | SideChainPart::Left => SideChainPart::Center,
    }
}

const fn side_chain_disconnected_from_right(part: SideChainPart) -> SideChainPart {
    match part {
        SideChainPart::Unconnected | SideChainPart::Left => SideChainPart::Unconnected,
        SideChainPart::Right | SideChainPart::Center => SideChainPart::Right,
    }
}

const fn side_chain_disconnected_from_left(part: SideChainPart) -> SideChainPart {
    match part {
        SideChainPart::Unconnected | SideChainPart::Right => SideChainPart::Unconnected,
        SideChainPart::Center | SideChainPart::Left => SideChainPart::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockHitResult;
    use pumpkin_data::BlockDirection;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn selectable_slot_container_shelf_and_bookshelf_columns() {
        let hit_left = BlockHitResult {
            face: &BlockDirection::South,
            cursor_pos: &Vector3::new(0.1, 0.8, 1.0),
        };
        let hit_mid = BlockHitResult {
            face: &BlockDirection::South,
            cursor_pos: &Vector3::new(0.5, 0.2, 1.0),
        };
        let hit_right = BlockHitResult {
            face: &BlockDirection::North,
            cursor_pos: &Vector3::new(0.1, 0.2, 0.0),
        };

        assert_eq!(
            hit_left.get_hit_slot(HorizontalFacing::South, 1, 3),
            Some(0)
        );
        assert_eq!(hit_mid.get_hit_slot(HorizontalFacing::South, 1, 3), Some(1));
        assert_eq!(
            hit_right.get_hit_slot(HorizontalFacing::North, 1, 3),
            Some(2)
        );
        // Chiseled bookshelf 2x3: top row (y=0.8 -> row 0), bottom row (y=0.2 -> row 1).
        assert_eq!(
            hit_left.get_hit_slot(HorizontalFacing::South, 2, 3),
            Some(0)
        );
        assert_eq!(hit_mid.get_hit_slot(HorizontalFacing::South, 2, 3), Some(4));
        assert_eq!(
            hit_right.get_hit_slot(HorizontalFacing::North, 2, 3),
            Some(5)
        );
        // Clicking the wrong face returns None.
        assert_eq!(hit_left.get_hit_slot(HorizontalFacing::North, 1, 3), None);
    }

    #[test]
    fn shelf_hotbar_slot_mapping_matches_vanilla() {
        // 1 powered shelf swaps hotbar slots 6, 7, 8.
        assert_eq!(shelf_hotbar_slot(1, 0, 0), Some(6));
        assert_eq!(shelf_hotbar_slot(1, 0, 2), Some(8));
        // 2 chained shelves swap hotbar slots 3..6 and 6..9.
        assert_eq!(shelf_hotbar_slot(2, 0, 0), Some(3));
        assert_eq!(shelf_hotbar_slot(2, 1, 2), Some(8));
        // 3 chained shelves swap hotbar slots 0..9.
        assert_eq!(shelf_hotbar_slot(3, 0, 0), Some(0));
        assert_eq!(shelf_hotbar_slot(3, 1, 1), Some(4));
        assert_eq!(shelf_hotbar_slot(3, 2, 2), Some(8));
    }

    #[test]
    fn side_chain_transitions_match_vanilla() {
        let left_end = side_chain_connected_to_right(SideChainPart::Unconnected);
        let right_end = side_chain_connected_to_left(SideChainPart::Unconnected);
        assert_eq!(left_end, SideChainPart::Left);
        assert_eq!(right_end, SideChainPart::Right);

        let center = side_chain_connected_to_right(right_end);
        assert_eq!(center, SideChainPart::Center);
        assert_eq!(
            side_chain_disconnected_from_right(center),
            SideChainPart::Right
        );
        assert_eq!(
            side_chain_disconnected_from_left(center),
            SideChainPart::Left
        );
        assert_eq!(
            side_chain_disconnected_from_right(SideChainPart::Left),
            SideChainPart::Unconnected
        );
    }
}
