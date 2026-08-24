use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    BlockProperties, EastRedstone, HorizontalFacing, NorthRedstone, ObserverLikeProperties,
    RedstoneWireLikeProperties, RepeaterLikeProperties, SouthRedstone, WestRedstone,
};
use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs,
    PrepareArgs,
};
use crate::world::World;

use super::get_redstone_power_no_dust;

type RedstoneWireProperties = RedstoneWireLikeProperties;

#[pumpkin_block("minecraft:redstone_wire")]
pub struct RedstoneWireBlock;

impl BlockBehaviour for RedstoneWireBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_survive(args.block_accessor, args.position)
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let initial_state = make_cross(0);
            let wire = get_connection_state(args.world, initial_state, args.position).await;
            wire.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            update_power_strength(args.world, args.position).await;

            for direction in [BlockDirection::Up, BlockDirection::Down] {
                let neighbor_pos = args.position.offset(direction.to_offset());
                update_neighbors_at(args.world, &neighbor_pos, &Block::REDSTONE_WIRE).await;
            }

            update_neighbors_of_neighboring_wires(args.world, args.position).await;
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            for direction in BlockDirection::all() {
                let neighbor_pos = args.position.offset(direction.to_offset());
                update_neighbors_at(args.world, &neighbor_pos, &Block::REDSTONE_WIRE).await;
            }

            update_neighbors_of_neighboring_wires(args.world, args.position).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                let (below_block, below_state) =
                    args.world.get_block_and_state(args.neighbor_position);
                if !can_survive_on(below_block, below_state) {
                    return Block::AIR.default_state.id;
                }
                return args.state_id;
            }

            let wire = RedstoneWireProperties::from_state_id(args.state_id, args.block);

            if args.direction == BlockDirection::Up {
                let new_wire = get_connection_state(args.world, wire, args.position).await;
                return new_wire.to_state_id(args.block);
            }

            let can_connect_up = !args
                .world
                .get_block_state(&args.position.up())
                .is_solid_block();
            let side_connection =
                get_connecting_side(args.world, args.position, args.direction, can_connect_up)
                    .await;

            let Some(horizontal) = args.direction.to_horizontal_facing() else {
                return args.state_id;
            };

            let current_side = get_side_connection(wire, horizontal);
            let is_connected_same = side_connection.is_connected() == current_side.is_connected();

            if is_connected_same && !is_cross(wire) {
                let mut new_wire = wire;
                set_side_connection(&mut new_wire, horizontal, side_connection);
                new_wire.to_state_id(args.block)
            } else {
                let mut cross = make_cross(wire.power);
                set_side_connection(&mut cross, horizontal, side_connection);
                let new_wire = get_connection_state(args.world, cross, args.position).await;
                new_wire.to_state_id(args.block)
            }
        })
    }

    fn prepare<'a>(&'a self, args: PrepareArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let wire = RedstoneWireProperties::from_state_id(args.state_id, args.block);

            for direction in BlockDirection::horizontal() {
                if is_side_connected_prop(wire, direction) {
                    let dir_block_pos = args.position.offset(direction.to_offset());
                    if args.world.get_block(&dir_block_pos) != &Block::REDSTONE_WIRE {
                        let down_pos = dir_block_pos.down();
                        if args.world.get_block(&down_pos) == &Block::REDSTONE_WIRE {
                            args.world
                                .replace_with_state_for_neighbor_update(
                                    &down_pos,
                                    direction.opposite().to_block_direction(),
                                    args.flags,
                                )
                                .await;
                        }

                        let up_pos = dir_block_pos.up();
                        if args.world.get_block(&up_pos) == &Block::REDSTONE_WIRE {
                            args.world
                                .replace_with_state_for_neighbor_update(
                                    &up_pos,
                                    direction.opposite().to_block_direction(),
                                    args.flags,
                                )
                                .await;
                        }
                    }
                }
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let wire = RedstoneWireProperties::from_state_id(state.id, args.block);

            if is_cross(wire) || is_dot(wire) {
                let mut new_wire = if is_cross(wire) {
                    RedstoneWireProperties::default(&Block::REDSTONE_WIRE)
                } else {
                    make_cross(wire.power)
                };
                new_wire.power = wire.power;
                new_wire = get_connection_state(args.world, new_wire, args.position).await;

                if wire != new_wire {
                    args.world
                        .set_block_state(
                            args.position,
                            new_wire.to_state_id(&Block::REDSTONE_WIRE),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;

                    for direction in BlockDirection::horizontal() {
                        let relative_pos = args.position.offset(direction.to_offset());
                        let old_connected = is_side_connected_prop(wire, direction);
                        let new_connected = is_side_connected_prop(new_wire, direction);

                        if old_connected != new_connected
                            && args.world.get_block_state(&relative_pos).is_solid_block()
                        {
                            args.world
                                .update_neighbors(
                                    &relative_pos,
                                    Some(direction.opposite().to_block_direction()),
                                )
                                .await;
                        }
                    }

                    return BlockActionResult::Success;
                }
            }

            BlockActionResult::Pass
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if can_survive(args.world.as_ref(), args.position) {
                update_power_strength(args.world, args.position).await;
            } else {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let wire = RedstoneWireProperties::from_state_id(args.state.id, args.block);
            if wire.power == 0 || args.direction == BlockDirection::Down {
                return 0;
            }
            if args.direction == BlockDirection::Up {
                return wire.power;
            }
            if let Some(horizontal) = args.direction.opposite().to_horizontal_facing()
                && is_side_connected_prop(wire, horizontal)
            {
                return wire.power;
            }
            0
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let wire = RedstoneWireProperties::from_state_id(args.state.id, args.block);
            if wire.power == 0 || args.direction == BlockDirection::Down {
                return 0;
            }
            if args.direction == BlockDirection::Up {
                return wire.power;
            }
            if let Some(horizontal) = args.direction.opposite().to_horizontal_facing()
                && is_side_connected_prop(wire, horizontal)
            {
                return wire.power;
            }
            0
        })
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let wire = RedstoneWireProperties::from_state_id(state_id, block);
        let new_wire = match rotation {
            Rotation::Rotate180 => RedstoneWireProperties {
                north: wire.south.to_wire_connection().to_north(),
                east: wire.west.to_wire_connection().to_east(),
                south: wire.north.to_wire_connection().to_south(),
                west: wire.east.to_wire_connection().to_west(),
                power: wire.power,
            },
            Rotation::CounterClockwise90 => RedstoneWireProperties {
                north: wire.east.to_wire_connection().to_north(),
                east: wire.south.to_wire_connection().to_east(),
                south: wire.west.to_wire_connection().to_south(),
                west: wire.north.to_wire_connection().to_west(),
                power: wire.power,
            },
            Rotation::Clockwise90 => RedstoneWireProperties {
                north: wire.west.to_wire_connection().to_north(),
                east: wire.north.to_wire_connection().to_east(),
                south: wire.east.to_wire_connection().to_south(),
                west: wire.south.to_wire_connection().to_west(),
                power: wire.power,
            },
            Rotation::None => wire,
        };
        BlockState::from_id(new_wire.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let wire = RedstoneWireProperties::from_state_id(state_id, block);
        let new_wire = match mirror {
            Mirror::LeftRight => RedstoneWireProperties {
                north: wire.south.to_wire_connection().to_north(),
                south: wire.north.to_wire_connection().to_south(),
                east: wire.east,
                west: wire.west,
                power: wire.power,
            },
            Mirror::FrontBack => RedstoneWireProperties {
                north: wire.north,
                south: wire.south,
                east: wire.west.to_wire_connection().to_east(),
                west: wire.east.to_wire_connection().to_west(),
                power: wire.power,
            },
            Mirror::None => wire,
        };
        BlockState::from_id(new_wire.to_state_id(block))
    }
}

// ---------------------------------------------------------------------------
// Evaluator Logic (matching DefaultRedstoneWireEvaluator)
// ---------------------------------------------------------------------------

pub async fn update_power_strength(world: &Arc<World>, pos: &BlockPos) {
    let (block, state) = world.get_block_and_state(pos);
    if block != &Block::REDSTONE_WIRE {
        return;
    }

    let mut wire = RedstoneWireProperties::from_state_id(state.id, block);
    let target_strength = calculate_target_strength(world, pos).await;

    if wire.power != target_strength {
        wire.power = target_strength;
        let new_state_id = wire.to_state_id(&Block::REDSTONE_WIRE);

        world
            .set_block_state(pos, new_state_id, BlockFlags::empty())
            .await;

        let mut to_update = Vec::with_capacity(7);
        to_update.push(*pos);
        for direction in BlockDirection::all() {
            to_update.push(pos.offset(direction.to_offset()));
        }

        for block_pos in to_update {
            update_neighbors_at(world, &block_pos, &Block::REDSTONE_WIRE).await;
        }
    }
}

async fn calculate_target_strength(world: &World, pos: &BlockPos) -> u8 {
    let block_signal = get_block_signal(world, pos).await;
    if block_signal == 15 {
        return 15;
    }
    let wire_signal = get_incoming_wire_signal(world, pos);
    block_signal.max(wire_signal)
}

async fn get_block_signal(world: &World, pos: &BlockPos) -> u8 {
    let mut max_signal = 0;
    for side in BlockDirection::all() {
        let neighbor_pos = pos.offset(side.to_offset());
        let (neighbor_block, neighbor_state) = world.get_block_and_state(&neighbor_pos);
        let signal =
            get_redstone_power_no_dust(neighbor_block, neighbor_state, world, neighbor_pos, side)
                .await;
        if signal == 15 {
            return 15;
        }
        max_signal = max_signal.max(signal);
    }
    max_signal
}

fn get_incoming_wire_signal(world: &World, pos: &BlockPos) -> u8 {
    let mut max_wire_signal = 0;
    let up_pos = pos.up();
    let is_up_conductor = world.get_block_state(&up_pos).is_solid_block();

    for direction in BlockDirection::horizontal() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let (neighbor_block, neighbor_state) = world.get_block_and_state(&neighbor_pos);

        // Same level
        if neighbor_block == &Block::REDSTONE_WIRE {
            let wire = RedstoneWireProperties::from_state_id(neighbor_state.id, neighbor_block);
            max_wire_signal = max_wire_signal.max(wire.power);
        }

        // Wire UP: if pos.up() is not a solid conductor, and neighbor is a solid conductor
        if !is_up_conductor && neighbor_state.is_solid_block() {
            let neighbor_up_pos = neighbor_pos.up();
            let (up_block, up_state) = world.get_block_and_state(&neighbor_up_pos);
            if up_block == &Block::REDSTONE_WIRE {
                let wire = RedstoneWireProperties::from_state_id(up_state.id, up_block);
                max_wire_signal = max_wire_signal.max(wire.power);
            }
        } else if !neighbor_state.is_solid_block() {
            // Wire DOWN: if neighbor is not a solid conductor
            let neighbor_down_pos = neighbor_pos.down();
            let (down_block, down_state) = world.get_block_and_state(&neighbor_down_pos);
            if down_block == &Block::REDSTONE_WIRE {
                let wire = RedstoneWireProperties::from_state_id(down_state.id, down_block);
                max_wire_signal = max_wire_signal.max(wire.power);
            }
        }
    }

    max_wire_signal.saturating_sub(1)
}

pub async fn update_neighbors_at(world: &Arc<World>, pos: &BlockPos, source_block: &Block) {
    for direction in BlockDirection::update_order() {
        let neighbor_pos = pos.offset(direction.to_offset());
        world.update_neighbor(&neighbor_pos, source_block).await;
    }
}

async fn check_corner_change_at(world: &Arc<World>, pos: &BlockPos) {
    if world.get_block(pos) == &Block::REDSTONE_WIRE {
        update_neighbors_at(world, pos, &Block::REDSTONE_WIRE).await;
        for direction in BlockDirection::all() {
            let neighbor_pos = pos.offset(direction.to_offset());
            update_neighbors_at(world, &neighbor_pos, &Block::REDSTONE_WIRE).await;
        }
    }
}

async fn update_neighbors_of_neighboring_wires(world: &Arc<World>, pos: &BlockPos) {
    for direction in BlockDirection::horizontal() {
        let neighbor_pos = pos.offset(direction.to_offset());
        check_corner_change_at(world, &neighbor_pos).await;
    }

    for direction in BlockDirection::horizontal() {
        let target = pos.offset(direction.to_offset());
        if world.get_block_state(&target).is_solid_block() {
            check_corner_change_at(world, &target.up()).await;
        } else {
            check_corner_change_at(world, &target.down()).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Connection & Shape Helper Functions
// ---------------------------------------------------------------------------

pub async fn get_connection_state(
    world: &World,
    state: RedstoneWireProperties,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    let was_dot = is_dot(state);
    let mut default_state = RedstoneWireProperties::default(&Block::REDSTONE_WIRE);
    default_state.power = state.power;

    let mut new_state = get_missing_connections(world, default_state, pos).await;
    if was_dot && is_dot(new_state) {
        return new_state;
    }

    let north = new_state.north.to_wire_connection().is_connected();
    let south = new_state.south.to_wire_connection().is_connected();
    let east = new_state.east.to_wire_connection().is_connected();
    let west = new_state.west.to_wire_connection().is_connected();

    let north_south_empty = !north && !south;
    let east_west_empty = !east && !west;

    if !west && north_south_empty {
        new_state.west = WestRedstone::Side;
    }
    if !east && north_south_empty {
        new_state.east = EastRedstone::Side;
    }
    if !north && east_west_empty {
        new_state.north = NorthRedstone::Side;
    }
    if !south && east_west_empty {
        new_state.south = SouthRedstone::Side;
    }

    new_state
}

async fn get_missing_connections(
    world: &World,
    mut state: RedstoneWireProperties,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    let can_connect_up = !world.get_block_state(&pos.up()).is_solid_block();

    for direction in BlockDirection::horizontal() {
        if !is_side_connected_prop(state, direction) {
            let side_connection =
                get_connecting_side(world, pos, direction.to_block_direction(), can_connect_up)
                    .await;
            set_side_connection(&mut state, direction, side_connection);
        }
    }

    state
}

async fn get_connecting_side(
    world: &World,
    pos: &BlockPos,
    direction: BlockDirection,
    can_connect_up: bool,
) -> WireConnection {
    let relative_pos = pos.offset(direction.to_offset());
    let (relative_block, relative_state) = world.get_block_and_state(&relative_pos);

    if can_connect_up {
        let is_placeable_above =
            is_trapdoor(relative_block) || can_survive_on(relative_block, relative_state);
        let (above_block, above_state) = world.get_block_and_state(&relative_pos.up());
        if is_placeable_above && should_connect_to(world, above_block, above_state, None).await {
            if relative_state.is_side_solid(direction.opposite()) {
                return WireConnection::Up;
            }
            return WireConnection::Side;
        }
    }

    let connects_to_relative =
        should_connect_to(world, relative_block, relative_state, Some(direction)).await;
    let (below_block, below_state) = world.get_block_and_state(&relative_pos.down());
    let connects_to_below = should_connect_to(world, below_block, below_state, None).await;

    if !connects_to_relative && (relative_state.is_solid_block() || !connects_to_below) {
        WireConnection::None
    } else {
        WireConnection::Side
    }
}

async fn should_connect_to(
    world: &World,
    block: &Block,
    state: &BlockState,
    direction: Option<BlockDirection>,
) -> bool {
    if block == &Block::REDSTONE_WIRE {
        return true;
    }
    if block == &Block::REPEATER {
        let repeater_props = RepeaterLikeProperties::from_state_id(state.id, block);
        let repeater_facing = repeater_props.facing.to_block_direction();
        return direction
            .is_some_and(|dir| repeater_facing == dir || repeater_facing == dir.opposite());
    }
    if block == &Block::OBSERVER {
        let observer_props = ObserverLikeProperties::from_state_id(state.id, block);
        let observer_facing = observer_props.facing;
        return direction.is_some_and(|dir| dir.to_facing() == observer_facing);
    }
    if let Some(dir) = direction {
        world
            .block_registry
            .emits_redstone_power(block, state, dir)
            .await
    } else {
        false
    }
}

fn is_trapdoor(block: &Block) -> bool {
    block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_TRAPDOORS)
}

fn can_survive(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let below = pos.down();
    let (below_block, below_state) = world.get_block_and_state(&below);
    can_survive_on(below_block, below_state)
}

fn can_survive_on(relative_block: &Block, relative_state: &BlockState) -> bool {
    relative_state.is_side_solid(BlockDirection::Up) || relative_block == &Block::HOPPER
}

#[must_use]
pub fn is_dot(wire: RedstoneWireProperties) -> bool {
    !wire.north.to_wire_connection().is_connected()
        && !wire.south.to_wire_connection().is_connected()
        && !wire.east.to_wire_connection().is_connected()
        && !wire.west.to_wire_connection().is_connected()
}

#[must_use]
pub fn is_cross(wire: RedstoneWireProperties) -> bool {
    wire.north.to_wire_connection().is_connected()
        && wire.south.to_wire_connection().is_connected()
        && wire.east.to_wire_connection().is_connected()
        && wire.west.to_wire_connection().is_connected()
}

#[must_use]
pub const fn make_cross(power: u8) -> RedstoneWireProperties {
    RedstoneWireProperties {
        north: NorthRedstone::Side,
        south: SouthRedstone::Side,
        east: EastRedstone::Side,
        west: WestRedstone::Side,
        power,
    }
}

fn get_side_connection(
    wire: RedstoneWireProperties,
    direction: HorizontalFacing,
) -> WireConnection {
    match direction {
        HorizontalFacing::North => wire.north.to_wire_connection(),
        HorizontalFacing::South => wire.south.to_wire_connection(),
        HorizontalFacing::East => wire.east.to_wire_connection(),
        HorizontalFacing::West => wire.west.to_wire_connection(),
    }
}

const fn set_side_connection(
    wire: &mut RedstoneWireProperties,
    direction: HorizontalFacing,
    connection: WireConnection,
) {
    match direction {
        HorizontalFacing::North => wire.north = connection.to_north(),
        HorizontalFacing::South => wire.south = connection.to_south(),
        HorizontalFacing::East => wire.east = connection.to_east(),
        HorizontalFacing::West => wire.west = connection.to_west(),
    }
}

fn is_side_connected_prop(wire: RedstoneWireProperties, direction: HorizontalFacing) -> bool {
    get_side_connection(wire, direction).is_connected()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireConnection {
    Up,
    Side,
    None,
}

impl WireConnection {
    #[must_use]
    pub const fn is_connected(self) -> bool {
        !matches!(self, Self::None)
    }

    #[must_use]
    pub const fn to_north(self) -> NorthRedstone {
        match self {
            Self::Up => NorthRedstone::Up,
            Self::Side => NorthRedstone::Side,
            Self::None => NorthRedstone::None,
        }
    }

    #[must_use]
    pub const fn to_south(self) -> SouthRedstone {
        match self {
            Self::Up => SouthRedstone::Up,
            Self::Side => SouthRedstone::Side,
            Self::None => SouthRedstone::None,
        }
    }

    #[must_use]
    pub const fn to_east(self) -> EastRedstone {
        match self {
            Self::Up => EastRedstone::Up,
            Self::Side => EastRedstone::Side,
            Self::None => EastRedstone::None,
        }
    }

    #[must_use]
    pub const fn to_west(self) -> WestRedstone {
        match self {
            Self::Up => WestRedstone::Up,
            Self::Side => WestRedstone::Side,
            Self::None => WestRedstone::None,
        }
    }
}

pub trait CardinalWireConnectionExt {
    fn to_wire_connection(&self) -> WireConnection;
}

impl CardinalWireConnectionExt for NorthRedstone {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }
}

impl CardinalWireConnectionExt for SouthRedstone {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }
}

impl CardinalWireConnectionExt for EastRedstone {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }
}

impl CardinalWireConnectionExt for WestRedstone {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }
}
