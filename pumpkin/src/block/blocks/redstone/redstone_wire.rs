use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, EastWireConnection, EnumVariants, Integer0To15, NorthWireConnection,
    ObserverLikeProperties, RedstoneWireLikeProperties, RepeaterLikeProperties,
    SouthWireConnection, WestWireConnection,
};
use pumpkin_data::{Block, BlockDirection, BlockState, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::pumpkin_block::{
    BrokenArgs, CanPlaceAtArgs, GetRedstonePowerArgs, GetStateForNeighborUpdateArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs, PrepareArgs, UseWithItemArgs,
};
use crate::block::registry::BlockActionResult;
use crate::{
    block::pumpkin_block::{NormalUseArgs, PumpkinBlock},
    world::World,
};

use super::turbo::RedstoneWireTurbo;
use super::{get_redstone_power_no_dust, update_wire_neighbors};

type RedstoneWireProperties = RedstoneWireLikeProperties;

#[pumpkin_block("minecraft:redstone_wire")]
pub struct RedstoneWireBlock;

#[async_trait]
impl PumpkinBlock for RedstoneWireBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut wire = RedstoneWireProperties::default(args.block);
        wire.power =
            Integer0To15::from_index(calculate_power(args.world, args.location).await.into());
        wire = get_regulated_sides(wire, args.world, args.location).await;
        if is_dot(wire) {
            wire = make_cross(wire.power);
        }

        wire.to_state_id(args.block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut wire = RedstoneWireProperties::from_state_id(args.state_id, args.block);
        let old_state = wire;
        let new_side: WireConnection;

        match args.direction {
            BlockDirection::Up => {
                return args.state_id;
            }
            BlockDirection::Down => {
                return get_regulated_sides(wire, args.world, args.location)
                    .await
                    .to_state_id(args.block);
            }
            BlockDirection::North => {
                let side = get_side(args.world, args.location, BlockDirection::North).await;
                wire.north = side.to_north();
                new_side = side;
            }
            BlockDirection::South => {
                let side = get_side(args.world, args.location, BlockDirection::South).await;
                wire.south = side.to_south();
                new_side = side;
            }
            BlockDirection::East => {
                let side = get_side(args.world, args.location, BlockDirection::East).await;
                wire.east = side.to_east();
                new_side = side;
            }
            BlockDirection::West => {
                let side = get_side(args.world, args.location, BlockDirection::West).await;
                wire.west = side.to_west();
                new_side = side;
            }
        }

        wire = get_regulated_sides(wire, args.world, args.location).await;
        if is_cross(old_state) && new_side.is_none() {
            return wire.to_state_id(args.block);
        }
        if !is_dot(old_state) && is_dot(wire) {
            let power = wire.power;
            wire = make_cross(power);
        }
        wire.to_state_id(args.block)
    }

    async fn prepare(&self, args: PrepareArgs<'_>) {
        let wire_props =
            RedstoneWireLikeProperties::from_state_id(args.state_id, &Block::REDSTONE_WIRE);

        for direction in BlockDirection::horizontal() {
            let other_block_pos = args.location.offset(direction.to_offset());
            let other_block = args.world.get_block(&other_block_pos).await;

            if wire_props.is_side_connected(direction) && other_block != &Block::REDSTONE_WIRE {
                let up_block_pos = other_block_pos.up();
                let up_block = args.world.get_block(&up_block_pos).await;
                if up_block == &Block::REDSTONE_WIRE {
                    args.world
                        .replace_with_state_for_neighbor_update(
                            &up_block_pos,
                            direction.opposite(),
                            args.flags,
                        )
                        .await;
                }

                let down_block_pos = other_block_pos.down();
                let down_block = args.world.get_block(&down_block_pos).await;
                if down_block == &Block::REDSTONE_WIRE {
                    args.world
                        .replace_with_state_for_neighbor_update(
                            &down_block_pos,
                            direction.opposite(),
                            args.flags,
                        )
                        .await;
                }
            }
        }
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let wire = RedstoneWireProperties::from_state_id(state.id, args.block);
        on_use(wire, args.world, args.location).await;
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.location).await;
        let wire = RedstoneWireProperties::from_state_id(state.id, args.block);
        if on_use(wire, args.world, args.location).await {
            BlockActionResult::Consume
        } else {
            BlockActionResult::Continue
        }
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if can_place_at(args.world.as_ref(), args.location).await {
            let state = args.world.get_block_state(args.location).await;
            let mut wire = RedstoneWireProperties::from_state_id(state.id, args.block);
            let new_power = calculate_power(args.world, args.location).await;
            if wire.power.to_index() as u8 != new_power {
                wire.power = Integer0To15::from_index(new_power.into());
                args.world
                    .set_block_state(
                        args.location,
                        wire.to_state_id(&Block::REDSTONE_WIRE),
                        BlockFlags::empty(),
                    )
                    .await;
                RedstoneWireTurbo::update_surrounding_neighbors(args.world, *args.location).await;
            }
        } else {
            args.world
                .break_block(args.location, None, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let wire = RedstoneWireProperties::from_state_id(args.state.id, args.block);
        if args.direction == BlockDirection::Up || wire.is_side_connected(args.direction.opposite())
        {
            wire.power.to_index() as u8
        } else {
            0
        }
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let wire = RedstoneWireProperties::from_state_id(args.state.id, args.block);
        if args.direction == BlockDirection::Up || wire.is_side_connected(args.direction.opposite())
        {
            wire.power.to_index() as u8
        } else {
            0
        }
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        update_wire_neighbors(args.world, args.location).await;
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        update_wire_neighbors(args.world, args.location).await;
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let floor = world.get_block_state(&block_pos.down()).await;
    floor.is_side_solid(BlockDirection::Up)
}

async fn on_use(wire: RedstoneWireProperties, world: &Arc<World>, block_pos: &BlockPos) -> bool {
    if is_cross(wire) || is_dot(wire) {
        let mut new_wire = if is_cross(wire) {
            RedstoneWireProperties::default(&Block::REDSTONE_WIRE)
        } else {
            make_cross(wire.power)
        };
        new_wire.power = wire.power;

        new_wire = get_regulated_sides(new_wire, world, block_pos).await;
        if wire != new_wire {
            world
                .set_block_state(
                    block_pos,
                    new_wire.to_state_id(&Block::REDSTONE_WIRE),
                    BlockFlags::empty(),
                )
                .await;
            update_wire_neighbors(world, block_pos).await;
            return true;
        }
    }
    false
}

#[must_use]
pub fn make_cross(power: Integer0To15) -> RedstoneWireProperties {
    RedstoneWireProperties {
        north: NorthWireConnection::Side,
        south: SouthWireConnection::Side,
        east: EastWireConnection::Side,
        west: WestWireConnection::Side,
        power,
    }
}

async fn can_connect_to(
    world: &World,
    block: &Block,
    side: BlockDirection,
    state: &BlockState,
) -> bool {
    if world
        .block_registry
        .emits_redstone_power(block, state, side)
        .await
    {
        return true;
    }
    if block == &Block::REPEATER {
        let repeater_props = RepeaterLikeProperties::from_state_id(state.id, block);
        return repeater_props.facing.to_block_direction() == side
            || repeater_props.facing.to_block_direction() == side.opposite();
    } else if block == &Block::OBSERVER {
        let observer_props = ObserverLikeProperties::from_state_id(state.id, block);
        return observer_props.facing == side.to_facing();
    } else if block == &Block::REDSTONE_WIRE {
        return true;
    }
    false
}

fn can_connect_diagonal_to(block: &Block) -> bool {
    block == &Block::REDSTONE_WIRE
}

pub async fn get_side(world: &World, pos: &BlockPos, side: BlockDirection) -> WireConnection {
    let neighbor_pos: BlockPos = pos.offset(side.to_offset());
    let (neighbor, state) = world.get_block_and_block_state(&neighbor_pos).await;

    if can_connect_to(world, neighbor, side, state).await {
        return WireConnection::Side;
    }

    let up_pos = pos.offset(BlockDirection::Up.to_offset());
    let up_state = world.get_block_state(&up_pos).await;

    if !up_state.is_solid()
        && can_connect_diagonal_to(
            world
                .get_block(&neighbor_pos.offset(BlockDirection::Up.to_offset()))
                .await,
        )
    {
        WireConnection::Up
    } else if !state.is_solid()
        && can_connect_diagonal_to(
            world
                .get_block(&neighbor_pos.offset(BlockDirection::Down.to_offset()))
                .await,
        )
    {
        WireConnection::Side
    } else {
        WireConnection::None
    }
}

async fn get_all_sides(
    mut wire: RedstoneWireProperties,
    world: &World,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    wire.north = get_side(world, pos, BlockDirection::North).await.to_north();
    wire.south = get_side(world, pos, BlockDirection::South).await.to_south();
    wire.east = get_side(world, pos, BlockDirection::East).await.to_east();
    wire.west = get_side(world, pos, BlockDirection::West).await.to_west();
    wire
}

#[must_use]
pub fn is_dot(wire: RedstoneWireProperties) -> bool {
    wire.north == NorthWireConnection::None
        && wire.south == SouthWireConnection::None
        && wire.east == EastWireConnection::None
        && wire.west == WestWireConnection::None
}

#[must_use]
pub fn is_cross(wire: RedstoneWireProperties) -> bool {
    wire.north == NorthWireConnection::Side
        && wire.south == SouthWireConnection::Side
        && wire.east == EastWireConnection::Side
        && wire.west == WestWireConnection::Side
}

pub async fn get_regulated_sides(
    wire: RedstoneWireProperties,
    world: &World,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    let mut state = get_all_sides(wire, world, pos).await;
    if is_dot(wire) && is_dot(state) {
        return state;
    }
    let north_none = state.north.is_none();
    let south_none = state.south.is_none();
    let east_none = state.east.is_none();
    let west_none = state.west.is_none();
    let north_south_none = north_none && south_none;
    let east_west_none = east_none && west_none;
    if north_none && east_west_none {
        state.north = NorthWireConnection::Side;
    }
    if south_none && east_west_none {
        state.south = SouthWireConnection::Side;
    }
    if east_none && north_south_none {
        state.east = EastWireConnection::Side;
    }
    if west_none && north_south_none {
        state.west = WestWireConnection::Side;
    }
    state
}

trait RedstoneWireLikePropertiesExt {
    fn is_side_connected(&self, direction: BlockDirection) -> bool;
    //fn get_connection_type(&self, direction: BlockDirection) -> WireConnection;
}

impl RedstoneWireLikePropertiesExt for RedstoneWireLikeProperties {
    fn is_side_connected(&self, direction: BlockDirection) -> bool {
        match direction {
            BlockDirection::North => self.north.to_wire_connection().is_connected(),
            BlockDirection::South => self.south.to_wire_connection().is_connected(),
            BlockDirection::East => self.east.to_wire_connection().is_connected(),
            BlockDirection::West => self.west.to_wire_connection().is_connected(),
            _ => false,
        }
    }

    /*
    fn get_connection_type(&self, direction: BlockDirection) -> WireConnection {
        match direction {
            BlockDirection::North => self.north.to_wire_connection(),
            BlockDirection::South => self.south.to_wire_connection(),
            BlockDirection::East => self.east.to_wire_connection(),
            BlockDirection::West => self.west.to_wire_connection(),
            _ => WireConnection::None,
        }
    }
     */
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireConnection {
    Up,
    Side,
    None,
}

impl WireConnection {
    fn is_connected(self) -> bool {
        self != Self::None
    }

    fn is_none(self) -> bool {
        self == Self::None
    }

    fn to_north(self) -> NorthWireConnection {
        match self {
            Self::Up => NorthWireConnection::Up,
            Self::Side => NorthWireConnection::Side,
            Self::None => NorthWireConnection::None,
        }
    }

    fn to_south(self) -> SouthWireConnection {
        match self {
            Self::Up => SouthWireConnection::Up,
            Self::Side => SouthWireConnection::Side,
            Self::None => SouthWireConnection::None,
        }
    }

    fn to_east(self) -> EastWireConnection {
        match self {
            Self::Up => EastWireConnection::Up,
            Self::Side => EastWireConnection::Side,
            Self::None => EastWireConnection::None,
        }
    }

    fn to_west(self) -> WestWireConnection {
        match self {
            Self::Up => WestWireConnection::Up,
            Self::Side => WestWireConnection::Side,
            Self::None => WestWireConnection::None,
        }
    }
}
trait CardinalWireConnectionExt {
    fn to_wire_connection(&self) -> WireConnection;
    fn is_none(&self) -> bool;
}

impl CardinalWireConnectionExt for NorthWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for SouthWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for EastWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for WestWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

async fn max_wire_power(wire_power: u8, world: &World, pos: BlockPos) -> u8 {
    let (block, block_state) = world.get_block_and_block_state(&pos).await;
    if block == &Block::REDSTONE_WIRE {
        let wire = RedstoneWireProperties::from_state_id(block_state.id, block);
        wire_power.max(wire.power.to_index() as u8)
    } else {
        wire_power
    }
}

async fn calculate_power(world: &World, pos: &BlockPos) -> u8 {
    let mut block_power: u8 = 0;
    let mut wire_power: u8 = 0;

    let up_pos = pos.offset(BlockDirection::Up.to_offset());
    let (_up_block, up_state) = world.get_block_and_block_state(&up_pos).await;

    for side in BlockDirection::all() {
        let neighbor_pos = pos.offset(side.to_offset());
        wire_power = max_wire_power(wire_power, world, neighbor_pos).await;
        let (neighbor, neighbor_state) = world.get_block_and_block_state(&neighbor_pos).await;
        block_power = block_power.max(
            get_redstone_power_no_dust(neighbor, neighbor_state, world, neighbor_pos, side).await,
        );
        if side.is_horizontal() {
            if !up_state.is_solid()
            /*TODO: &&  !neighbor.is_transparent() */
            {
                wire_power = max_wire_power(
                    wire_power,
                    world,
                    neighbor_pos.offset(BlockDirection::Up.to_offset()),
                )
                .await;
            }

            if !neighbor_state.is_solid() {
                wire_power = max_wire_power(
                    wire_power,
                    world,
                    neighbor_pos.offset(BlockDirection::Down.to_offset()),
                )
                .await;
            }
        }
    }

    block_power.max(wire_power.saturating_sub(1))
}
