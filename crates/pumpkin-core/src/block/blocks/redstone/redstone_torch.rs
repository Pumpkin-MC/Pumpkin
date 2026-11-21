use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use crate::block::BlockIsReplacing;
use crate::block::CanPlaceAtArgs;
use crate::block::EmitsRedstonePowerArgs;
use crate::block::GetRedstonePowerArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::entity::EntityBase;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::FacingExt;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

pub const RECENT_TOGGLE_TIMER: u64 = 60;
pub const MAX_RECENT_TOGGLES: usize = 8;
pub const RESTART_DELAY: u8 = 160;
pub const TOGGLE_DELAY: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TorchKey {
    world_id: usize,
    pos: BlockPos,
}

static RECENT_TOGGLES: Mutex<Option<FxHashMap<TorchKey, Vec<u64>>>> = Mutex::new(None);

fn is_toggled_too_frequently(world: &World, pos: BlockPos, add: bool) -> bool {
    let world_id = std::ptr::from_ref(world) as usize;
    let game_time = world.get_world_age() as u64;
    is_toggled_too_frequently_raw(world_id, game_time, pos, add)
}

fn is_toggled_too_frequently_raw(
    world_id: usize,
    game_time: u64,
    pos: BlockPos,
    add: bool,
) -> bool {
    let key = TorchKey { world_id, pos };
    let mut guard = RECENT_TOGGLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let map = guard.get_or_insert_with(FxHashMap::default);
    let toggles = map.entry(key).or_default();
    toggles.retain(|&time| game_time.saturating_sub(time) <= RECENT_TOGGLE_TIMER);
    if add {
        toggles.push(game_time);
    }
    toggles.len() >= MAX_RECENT_TOGGLES
}

fn clear_recent_toggles_raw(world_id: usize, pos: BlockPos) {
    let key = TorchKey { world_id, pos };
    let mut guard = RECENT_TOGGLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(map) = guard.as_mut() {
        map.remove(&key);
    }
}

fn clear_recent_toggles(world: &World, pos: BlockPos) {
    let world_id = std::ptr::from_ref(world) as usize;
    clear_recent_toggles_raw(world_id, pos);
}

type RWallTorchProps = pumpkin_data::block_properties::FurnaceLikeProperties;
type RTorchProps = pumpkin_data::block_properties::RedstoneOreLikeProperties;

use crate::block::{BlockBehaviour, BlockMetadata};
use crate::world::World;

use super::get_redstone_power;

pub struct RedstoneTorchBlock;

impl BlockMetadata for RedstoneTorchBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::REDSTONE_TORCH, BlockId::REDSTONE_WALL_TORCH].into()
    }
}

impl BlockBehaviour for RedstoneTorchBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let world = args.world;
        let block = args.block;
        let location = args.position;

        if args.direction == BlockDirection::Down {
            let support_block = world.get_block_state(&location.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state.id;
            }
        }
        let mut directions = args.player.get_entity().get_entity_facing_order();

        if args.replacing == BlockIsReplacing::None {
            let face = args.direction.to_facing();
            let mut i = 0;
            while i < directions.len() && directions[i] != face {
                i += 1;
            }

            if i > 0 {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        } else if directions[0] == Facing::Down {
            let support_block = world.get_block_state(&location.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state.id;
            }
        }

        for dir in directions {
            if dir != Facing::Up
                && dir != Facing::Down
                && can_place_at(world, location, dir.to_block_direction())
            {
                let mut torch_props = RWallTorchProps::default(&Block::REDSTONE_WALL_TORCH);
                if let Some(facing) = dir.opposite().to_horizontal_facing() {
                    torch_props.facing = facing;
                    return torch_props.to_state_id(&Block::REDSTONE_WALL_TORCH);
                }
            }
        }

        let support_block = world.get_block_state(&location.down());
        if support_block.is_center_solid(BlockDirection::Up) {
            block.default_state.id
        } else {
            BlockStateId::AIR
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let support_block = args.block_accessor.get_block_state(&args.position.down());
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(args.block_accessor, args.position, dir.to_block_direction()) {
                return true;
            }
        }
        false
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(args.state_id);
            if props.facing.to_block_direction().opposite() == args.direction
                && !can_place_at(
                    args.world,
                    args.position,
                    props.facing.to_block_direction().opposite(),
                )
            {
                return BlockStateId::AIR;
            }
        } else if args.direction == BlockDirection::Down {
            let support_block = args.world.get_block_state(&args.position.down());
            if !support_block.is_center_solid(BlockDirection::Up) {
                return BlockStateId::AIR;
            }
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state = args.world.get_block_state(args.position);

            if args
                .world
                .is_block_tick_scheduled(args.position, args.block)
            {
                return;
            }

            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(state.id);
                if props.lit
                    != should_be_lit(
                        args.world,
                        args.position,
                        props.facing.to_block_direction().opposite(),
                    )
                {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        TOGGLE_DELAY,
                        TickPriority::Normal,
                    );
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(state.id);
                if props.lit != should_be_lit(args.world, args.position, BlockDirection::Down) {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        TOGGLE_DELAY,
                        TickPriority::Normal,
                    );
                }
            }
        }
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(args.state.id);
            if props.lit && args.direction != props.facing.to_block_direction() {
                return 15;
            }
        } else if args.block == &Block::REDSTONE_TORCH {
            let props = RTorchProps::from_state_id(args.state.id);
            if props.lit && args.direction != BlockDirection::Up {
                return 15;
            }
        }
        0
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.direction == BlockDirection::Down {
            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(args.state.id);
                if props.lit {
                    return 15;
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(args.state.id);
                if props.lit {
                    return 15;
                }
            }
        }
        0
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(args.position);
        if block == &Block::REDSTONE_WALL_TORCH {
            let mut props = RWallTorchProps::from_state_id(state.id);
            let should_be_lit_now = should_be_lit(
                args.world,
                args.position,
                props.facing.to_block_direction().opposite(),
            );
            if props.lit {
                if !should_be_lit_now {
                    props.lit = false;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    );
                    update_neighbors(args.world, args.position);
                    if is_toggled_too_frequently(args.world, *args.position, true) {
                        args.world.sync_world_event(
                            WorldEvent::RedstoneTorchBurnout,
                            *args.position,
                            0,
                        );
                        args.world.schedule_block_tick(
                            block,
                            *args.position,
                            RESTART_DELAY,
                            TickPriority::Normal,
                        );
                    }
                }
            } else if should_be_lit_now
                && !is_toggled_too_frequently(args.world, *args.position, false)
            {
                props.lit = true;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                );
                update_neighbors(args.world, args.position);
            }
        } else if block == &Block::REDSTONE_TORCH {
            let mut props = RTorchProps::from_state_id(state.id);
            let should_be_lit_now = should_be_lit(args.world, args.position, BlockDirection::Down);
            if props.lit {
                if !should_be_lit_now {
                    props.lit = false;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    );
                    update_neighbors(args.world, args.position);
                    if is_toggled_too_frequently(args.world, *args.position, true) {
                        args.world.sync_world_event(
                            WorldEvent::RedstoneTorchBurnout,
                            *args.position,
                            0,
                        );
                        args.world.schedule_block_tick(
                            block,
                            *args.position,
                            RESTART_DELAY,
                            TickPriority::Normal,
                        );
                    }
                }
            } else if should_be_lit_now
                && !is_toggled_too_frequently(args.world, *args.position, false)
            {
                props.lit = true;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                );
                update_neighbors(args.world, args.position);
            }
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        update_neighbors(args.world, args.position);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            clear_recent_toggles(args.world, *args.position);
            update_neighbors(args.world, args.position);
        }
    }
}

pub fn should_be_lit(world: &World, pos: &BlockPos, face: BlockDirection) -> bool {
    let other_pos = pos.offset(face.to_offset());
    let (block, state) = world.get_block_and_state(&other_pos);
    get_redstone_power(block, state, world, &other_pos, face) == 0
}

pub fn update_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for dir in BlockDirection::all() {
        let other_pos = pos.offset(dir.to_offset());
        world.update_neighbors(&other_pos, None);
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .is_side_solid(facing.opposite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torch_burnout_requires_eight_toggles_in_sixty_ticks() {
        let world_id = 99999;
        let pos = BlockPos::new(10, 64, 10);
        clear_recent_toggles_raw(world_id, pos);

        for tick in 1..=7 {
            assert!(
                !is_toggled_too_frequently_raw(world_id, tick, pos, true),
                "toggle {tick} should not cause burnout"
            );
        }

        assert!(
            is_toggled_too_frequently_raw(world_id, 8, pos, true),
            "8th toggle within 60 ticks must cause burnout"
        );

        assert!(
            is_toggled_too_frequently_raw(world_id, 10, pos, false),
            "must remain burned out while toggles are recent"
        );

        assert!(
            !is_toggled_too_frequently_raw(world_id, 80, pos, false),
            "toggles older than 60 ticks must expire"
        );

        clear_recent_toggles_raw(world_id, pos);
    }
}
