use std::sync::{Arc, LazyLock, Mutex};

use rustc_hash::FxHashMap;

use crate::block::BlockFuture;
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
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type RWallTorchProps = pumpkin_data::block_properties::FurnaceLikeProperties;
type RTorchProps = pumpkin_data::block_properties::RedstoneOreLikeProperties;

use crate::block::{BlockBehaviour, BlockMetadata};
use crate::world::World;

use super::get_redstone_power;

/// Vanilla `RedstoneTorchBlock.RECENT_TOGGLE_TIMER` (`RedstoneTorchBlock.java:39`):
/// toggle records older than this many ticks are pruned.
const RECENT_TOGGLE_TIMER: i64 = 60;
/// Vanilla `RedstoneTorchBlock.MAX_RECENT_TOGGLES` (`RedstoneTorchBlock.java:40`):
/// the torch burns out at this many toggles inside the timer window.
const MAX_RECENT_TOGGLES: usize = 8;
/// Vanilla `RedstoneTorchBlock.RESTART_DELAY` (`RedstoneTorchBlock.java:41`):
/// ticks before a burned-out torch is re-checked.
const RESTART_DELAY: u8 = 160;

/// Vanilla `RedstoneTorchBlock.RECENT_TOGGLES` (`RedstoneTorchBlock.java:38`):
/// `WeakHashMap<BlockGetter, List<Toggle>>` — an ordered per-level list of
/// `(pos, game time)` toggle records. Pumpkin's `World` carries no generic
/// per-block map, so the equivalent lives here keyed by dimension id (Pumpkin
/// runs one world per dimension). Guarded by a sync mutex; never held across
/// an await point.
static RECENT_TOGGLES: LazyLock<Mutex<FxHashMap<u8, Vec<(BlockPos, i64)>>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

/// Vanilla `RedstoneTorchBlock.tick` (`RedstoneTorchBlock.java:79-82`): drop
/// toggle records older than `RECENT_TOGGLE_TIMER` from the front of the
/// per-level list.
fn prune_recent_toggles(world: &World, game_time: i64) {
    let mut map = RECENT_TOGGLES.lock().expect("RECENT_TOGGLES poisoned");
    if let Some(toggles) = map.get_mut(&world.dimension.id) {
        while !toggles.is_empty() && game_time - toggles[0].1 > RECENT_TOGGLE_TIMER {
            toggles.remove(0);
        }
    }
}

/// Vanilla `RedstoneTorchBlock.isToggledTooFrequently`
/// (`RedstoneTorchBlock.java:142-153`): optionally record the current toggle,
/// then report burnout once `MAX_RECENT_TOGGLES` records exist for this
/// position.
fn is_toggled_too_frequently(world: &World, pos: &BlockPos, game_time: i64, add: bool) -> bool {
    let mut map = RECENT_TOGGLES.lock().expect("RECENT_TOGGLES poisoned");
    let toggles = map.entry(world.dimension.id).or_default();
    if add {
        toggles.push((*pos, game_time));
    }
    // RedstoneTorchBlock.java:147-152: burned out once the list holds
    // `MAX_RECENT_TOGGLES` records for this position.
    toggles
        .iter()
        .filter(|(toggle_pos, _)| toggle_pos == pos)
        .nth(MAX_RECENT_TOGGLES - 1)
        .is_some()
}

pub struct RedstoneTorchBlock;

impl BlockMetadata for RedstoneTorchBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::REDSTONE_TORCH, BlockId::REDSTONE_WALL_TORCH].into()
    }
}

impl BlockBehaviour for RedstoneTorchBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
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
                    torch_props.facing = dir.opposite().to_horizontal_facing().unwrap();
                    return torch_props.to_state_id(&Block::REDSTONE_WALL_TORCH);
                }
            }

            let support_block = world.get_block_state(&location.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                block.default_state.id
            } else {
                BlockStateId::AIR
            }
        })
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

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Water/lava in cell → AIR; `replace_with_state_for_neighbor_update`
            // maps AIR to `break_block` + NOTIFY so clients drop ghost torches.
            // Fluid flow also `break_block` before placing water (PistonBehavior::Destroy).
            if redstone_torch_blocked_by_fluid(args.world, args.position) {
                return BlockStateId::AIR;
            }
            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(args.state_id, args.block);
                // Vanilla: directionToNeighbour.getOpposite() == FACING && !canSurvive
                let facing = props.facing.to_block_direction();
                if args.direction.opposite() == facing
                    && !can_place_at(args.world, args.position, facing.opposite())
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
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);

            // Fluid washed into cell (or residual after partial replace) → full notify break.
            if redstone_torch_blocked_by_fluid(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
                return;
            }

            // Support gone → break with full notify (client + cascade).
            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(state.id, args.block);
                let attach = props.facing.to_block_direction().opposite();
                if !can_place_at(args.world.as_ref(), args.position, attach) {
                    args.world
                        .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                        .await;
                    return;
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let support = args.world.get_block_state(&args.position.down());
                if !support.is_center_solid(BlockDirection::Up) {
                    args.world
                        .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                        .await;
                    return;
                }
            }

            if args
                .world
                .is_block_tick_scheduled(args.position, args.block)
            {
                return;
            }

            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(state.id, args.block);
                if props.lit
                    != should_be_lit(
                        args.world,
                        args.position,
                        props.facing.to_block_direction().opposite(),
                    )
                    .await
                {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        2,
                        TickPriority::Normal,
                    );
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(state.id, args.block);
                if props.lit != should_be_lit(args.world, args.position, BlockDirection::Down).await
                {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        2,
                        TickPriority::Normal,
                    );
                }
            }
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(args.state.id, args.block);
                if props.lit && args.direction != props.facing.to_block_direction() {
                    return 15;
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(args.state.id, args.block);
                if props.lit && args.direction != BlockDirection::Up {
                    return 15;
                }
            }
            0
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                if args.block == &Block::REDSTONE_WALL_TORCH {
                    let props = RWallTorchProps::from_state_id(args.state.id, args.block);
                    if props.lit {
                        return 15;
                    }
                } else if args.block == &Block::REDSTONE_TORCH {
                    let props = RTorchProps::from_state_id(args.state.id, args.block);
                    if props.lit {
                        return 15;
                    }
                }
            }
            0
        })
    }

    /// Vanilla `RedstoneTorchBlock.tick` (`RedstoneTorchBlock.java:77-94`).
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let (lit, input_face) = if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(state.id, args.block);
                (props.lit, props.facing.to_block_direction().opposite())
            } else {
                let props = RTorchProps::from_state_id(state.id, args.block);
                (props.lit, BlockDirection::Down)
            };
            // `hasNeighborSignal` (RedstoneTorchBlock.java:72-74,78).
            let has_neighbor_signal = !should_be_lit(args.world, args.position, input_face).await;
            let game_time = args.world.level_time.lock().await.query_gametime();
            // RedstoneTorchBlock.java:79-82: prune stale toggle records.
            prune_recent_toggles(args.world, game_time);
            if lit {
                if has_neighbor_signal {
                    // RedstoneTorchBlock.java:85: turn off (setBlock flag 3).
                    set_lit(args.world, args.block, args.position, state.id, false).await;
                    // RedstoneTorchBlock.java:86-89: record the toggle; on the
                    // 8th within 60 ticks the torch burns out — levelEvent 1502
                    // (fizz sound + smoke particles on the client) and a
                    // re-check after RESTART_DELAY ticks.
                    if is_toggled_too_frequently(args.world, args.position, game_time, true) {
                        args.world.sync_world_event(
                            WorldEvent::RedstoneTorchBurnout,
                            *args.position,
                            0,
                        );
                        args.world.schedule_block_tick(
                            args.block,
                            *args.position,
                            RESTART_DELAY,
                            TickPriority::Normal,
                        );
                    }
                }
            } else if !has_neighbor_signal
                && !is_toggled_too_frequently(args.world, args.position, game_time, false)
            {
                // RedstoneTorchBlock.java:91-93: relight only when not burned out.
                set_lit(args.world, args.block, args.position, state.id, true).await;
            }
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            update_neighbors(args.world, args.position).await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            update_neighbors(args.world, args.position).await;
        })
    }
}

/// Sets the LIT property and notifies neighbors. Vanilla's `setBlock(..., 3)`
/// re-runs `RedstoneTorchBlock.onPlace` → `notifyNeighbors`
/// (`RedstoneTorchBlock.java:53-63`); Pumpkin notifies explicitly.
async fn set_lit(
    world: &Arc<World>,
    block: &Block,
    pos: &BlockPos,
    state_id: BlockStateId,
    lit: bool,
) {
    let new_state_id = if block == &Block::REDSTONE_WALL_TORCH {
        let mut props = RWallTorchProps::from_state_id(state_id, block);
        props.lit = lit;
        props.to_state_id(block)
    } else {
        let mut props = RTorchProps::from_state_id(state_id, block);
        props.lit = lit;
        props.to_state_id(block)
    };
    world
        .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL)
        .await;
    update_neighbors(world, pos).await;
}

pub async fn should_be_lit(world: &World, pos: &BlockPos, face: BlockDirection) -> bool {
    let other_pos = pos.offset(face.to_offset());
    let (block, state) = world.get_block_and_state(&other_pos);
    get_redstone_power(block, state, world, &other_pos, face).await == 0
}

pub async fn update_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for dir in BlockDirection::all() {
        let other_pos = pos.offset(dir.to_offset());
        world.update_neighbors(&other_pos, None).await;
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .is_side_solid(facing.opposite())
}

/// True when this cell is water/lava (torch cannot remain).
fn redstone_torch_blocked_by_fluid(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let state = world.get_block_state(pos);
    if state.is_liquid() || state.is_waterlogged() {
        return true;
    }
    Fluid::from_state_id(state.id).is_some_and(|f| f.id != Fluid::EMPTY.id)
}
