use std::sync::atomic::AtomicBool;

use crate::block::redstone_view::get_received_redstone_power;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::{
    Block, BlockState, EastWireConnection, EnumVariants, Integer0To15, NorthWireConnection,
    ObserverLikeProperties, RedstoneWireLikeProperties, RepeaterLikeProperties,
    SouthWireConnection, WestWireConnection,
};
use pumpkin_data::block::{BlockProperties, HorizontalFacing};
use pumpkin_data::tag::Tagable;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::get_state_by_state_id;
use pumpkin_world::block::{BlockDirection, FacingExt, HorizontalFacingExt};

use crate::{block::pumpkin_block::PumpkinBlock, server::Server, world::World};

/// This is a bit confusing but dot state is actually the X shape
const DOT_STATE: RedstoneWireLikeProperties = RedstoneWireLikeProperties {
    power: Integer0To15::L0,
    east: EastWireConnection::Side,
    north: NorthWireConnection::Side,
    south: SouthWireConnection::Side,
    west: WestWireConnection::Side,
};

#[pumpkin_block("minecraft:redstone_wire")]
pub struct RedstoneWireBlock {
    // This is only instantiated once, so it's global
    pub wire_gives_power: AtomicBool,
}

#[async_trait]
impl PumpkinBlock for RedstoneWireBlock {
    // Start of placement

    async fn can_place_at(&self, world: &World, block_pos: &BlockPos) -> bool {
        let floor = world.get_block_state(&block_pos.down()).await.unwrap();
        Self::can_run_on_top(&floor)
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &HorizontalFacing,
        _other: bool,
    ) -> u16 {
        let wire_props = Self::get_placement_state(world, block_pos, DOT_STATE).await;

        wire_props.to_state_id(block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state_id: u16,
        block_pos: &BlockPos,
        direction: &BlockDirection,
        neighbor_pos: &BlockPos,
        _neighbor_state: u16,
    ) -> u16 {
        if direction == &BlockDirection::Down {
            let floor = world.get_block_state(neighbor_pos).await.unwrap();
            if !Self::can_run_on_top(&floor) {
                return Block::AIR.default_state_id;
            }
        } else if direction == &BlockDirection::Up {
            let placement_state = Self::get_placement_state(
                world,
                block_pos,
                RedstoneWireLikeProperties::from_state_id(state_id, &Block::REDSTONE_WIRE),
            )
            .await;

            return placement_state.to_state_id(&Block::REDSTONE_WIRE);
        } else {
            let mut wire_props =
                RedstoneWireLikeProperties::from_state_id(state_id, &Block::REDSTONE_WIRE);
            let block_above = world.get_block_state(&block_pos.up()).await.unwrap();
            let wire_connection_type = Self::get_render_connection_type(
                world,
                *block_pos,
                *direction,
                !block_above.is_solid,
            )
            .await;

            if wire_connection_type.is_connected()
                == wire_props.get_connection_type(*direction).is_connected()
                && !wire_props.is_fully_connected()
            {
                wire_connection_type.set_connection(&mut wire_props, *direction);

                return wire_props.to_state_id(&Block::REDSTONE_WIRE);
            }

            let mut new_props = DOT_STATE;
            new_props.power = wire_props.power;
            wire_connection_type.set_connection(&mut new_props, *direction);
            new_props = Self::get_placement_state(world, block_pos, new_props).await;

            return new_props.to_state_id(&Block::REDSTONE_WIRE);
        }

        state_id
    }

    async fn prepare(
        &self,
        world: &World,
        block_pos: &BlockPos,
        _block: &Block,
        state_id: u16,
        flags: BlockFlags,
    ) {
        let wire_props = if state_id == 0 {
            RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE)
        } else {
            RedstoneWireLikeProperties::from_state_id(state_id, &Block::REDSTONE_WIRE)
        };

        for direction in BlockDirection::horizontal() {
            let other_block_pos = block_pos.offset(direction.to_offset());
            let other_block = world.get_block(&other_block_pos).await.unwrap();

            if wire_props.is_side_connected(direction) && other_block != Block::REDSTONE_WIRE {
                let up_block_pos = other_block_pos.up();
                let up_block = world.get_block(&up_block_pos).await.unwrap();
                if up_block == Block::REDSTONE_WIRE {
                    world
                        .replace_with_state_for_neighbor_update(
                            &up_block_pos,
                            &direction.opposite(),
                            flags,
                        )
                        .await;
                }

                let down_block_pos = other_block_pos.down();
                let down_block = world.get_block(&down_block_pos).await.unwrap();
                if down_block == Block::REDSTONE_WIRE {
                    world
                        .replace_with_state_for_neighbor_update(
                            &down_block_pos,
                            &direction.opposite(),
                            flags,
                        )
                        .await;
                }
            }
        }
    }

    // End of placement, start of power calculation

    async fn placed(
        &self,
        world: &World,
        block: &Block,
        _state_id: u16,
        block_pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        let state = world.get_block_state(block_pos).await.unwrap();
        crate::block::redstone_controller::update(world, block_pos, block, &state, None, true)
            .await;

        for direction in BlockDirection::vertical() {
            world
                .update_neighbors(&block_pos.offset(direction.to_offset()), None)
                .await;
        }

        Self::update_offset_neighbors(world, block_pos).await;
    }

    async fn on_state_replaced(
        &self,
        world: &World,
        block: &Block,
        location: BlockPos,
        old_state_id: u16,
        moved: bool,
    ) {
        if !moved {
            for direction in BlockDirection::all() {
                world
                    .update_neighbors(&location.offset(direction.to_offset()), None)
                    .await;
            }

            let state = get_state_by_state_id(old_state_id).unwrap();

            crate::block::redstone_controller::update(world, &location, block, &state, None, false)
                .await;
            Self::update_offset_neighbors(world, &location).await;
        }
    }

    async fn on_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let block_state = world.get_block_state(block_pos).await.unwrap();

        if self.can_place_at(world, block_pos).await {
            crate::block::redstone_controller::update(
                world,
                block_pos,
                block,
                &block_state,
                None,
                false,
            )
            .await;
        } else {
            // TODO: Break the block with drops
            world
                .set_block_state(
                    block_pos,
                    Block::AIR.default_state_id,
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
        }
    }

    async fn emits_redstone_power(&self, _block: &Block, _state: &BlockState) -> bool {
        self.wire_gives_power
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        block_pos: &BlockPos,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> u8 {
        if self
            .wire_gives_power
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.get_weak_redstone_power(block, world, block_pos, state, direction)
                .await
        } else {
            0
        }
    }

    async fn get_weak_redstone_power(
        &self,
        _block: &Block,
        world: &World,
        block_pos: &BlockPos,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> u8 {
        let wire_gives_power = self
            .wire_gives_power
            .load(std::sync::atomic::Ordering::Relaxed);
        if wire_gives_power && *direction != BlockDirection::Down {
            let wire_props =
                RedstoneWireLikeProperties::from_state_id(state.id, &Block::REDSTONE_WIRE);
            let power = wire_props.power.to_index() as u8;

            if power == 0 {
                return 0;
            }

            let placement_state = Self::get_placement_state(world, block_pos, wire_props).await;

            if *direction != BlockDirection::Up
                && !placement_state
                    .get_connection_type(direction.opposite())
                    .is_connected()
            {
                return 0;
            }

            return power;
        }

        0
    }

    async fn get_strong_power(&self, world: &World, block_pos: &BlockPos) -> u8 {
        self.wire_gives_power
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let power = get_received_redstone_power(world, block_pos).await;
        self.wire_gives_power
            .store(true, std::sync::atomic::Ordering::Relaxed);
        power
    }
}

impl RedstoneWireBlock {
    async fn update_neighbors(world: &World, block_pos: &BlockPos) {
        let block = world.get_block(block_pos).await.unwrap();
        if block == Block::REDSTONE_WIRE {
            world.update_neighbors(block_pos, None).await;
            for direction in BlockDirection::all() {
                world
                    .update_neighbors(&block_pos.offset(direction.to_offset()), None)
                    .await;
            }
        }
    }

    async fn update_offset_neighbors(world: &World, block_pos: &BlockPos) {
        for direction in BlockDirection::horizontal() {
            Self::update_neighbors(world, &block_pos.offset(direction.to_offset())).await;
        }

        for direction in BlockDirection::horizontal() {
            let other_pos = block_pos.offset(direction.to_offset());
            let other_state = world.get_block_state(&other_pos).await.unwrap();

            if other_state.is_solid {
                Self::update_neighbors(world, &other_pos.up()).await;
            } else {
                Self::update_neighbors(world, &other_pos.down()).await;
            }
        }
    }

    pub async fn connects_to(
        world: &World,
        state: &BlockState,
        direction: Option<BlockDirection>,
    ) -> bool {
        let block = Block::from_state_id(state.id).unwrap();

        if block == Block::REDSTONE_WIRE {
            return true;
        } else if block == Block::REPEATER {
            if let Some(direction) = direction {
                let repeater_props =
                    RepeaterLikeProperties::from_state_id(state.id, &Block::REPEATER);

                return repeater_props.facing.to_block_direction() == direction
                    || repeater_props.facing.to_block_direction() == direction.opposite();
            }
        } else if block == Block::OBSERVER {
            if let Some(direction) = direction {
                let observer_props =
                    ObserverLikeProperties::from_state_id(state.id, &Block::OBSERVER);

                return observer_props.facing.to_block_direction() == direction;
            }
        } else if direction.is_some()
            && world
                .block_registry
                .emits_redstone_power(&block, state)
                .await
        {
            return true;
        }

        false
    }

    fn can_run_on_top(floor: &BlockState) -> bool {
        // TODO: Only check if top face is solid
        floor.is_solid
    }

    async fn get_render_connection_type(
        world: &World,
        location: BlockPos,
        direction: BlockDirection,
        not_solid: bool,
    ) -> WireConnection {
        let other_block_pos = location.offset(direction.to_offset());
        let (other_block, other_block_state) = world
            .get_block_and_block_state(&other_block_pos)
            .await
            .unwrap();

        if not_solid {
            let can_run_on_top = other_block.is_tagged_with("minecraft:trapdoors").unwrap()
                || Self::can_run_on_top(&other_block_state);

            let connects_up = Self::connects_to(
                world,
                &world.get_block_state(&other_block_pos.up()).await.unwrap(),
                None,
            )
            .await;

            if can_run_on_top && connects_up {
                // TODO: Check if side is solid instead
                if other_block_state.is_solid {
                    return WireConnection::Up;
                }
                return WireConnection::Side;
            }
        }

        if !Self::connects_to(world, &other_block_state, Some(direction)).await
            && (other_block_state.is_solid
                || !Self::connects_to(
                    world,
                    &world
                        .get_block_state(&other_block_pos.down())
                        .await
                        .unwrap(),
                    None,
                )
                .await)
        {
            return WireConnection::None;
        }

        WireConnection::Side
    }

    async fn get_default_wire_state(
        world: &World,
        block_pos: &BlockPos,
        props: RedstoneWireLikeProperties,
    ) -> RedstoneWireLikeProperties {
        let mut props = props;
        let not_solid = !world
            .get_block_state(&block_pos.up())
            .await
            .unwrap()
            .is_solid;

        for direction in BlockDirection::horizontal() {
            if !props.is_side_connected(direction) {
                let connection_type =
                    Self::get_render_connection_type(world, *block_pos, direction, not_solid).await;
                connection_type.set_connection(&mut props, direction);
            }
        }

        props
    }

    async fn get_placement_state(
        world: &World,
        block_pos: &BlockPos,
        old_props: RedstoneWireLikeProperties,
    ) -> RedstoneWireLikeProperties {
        let not_connected = old_props.is_not_connected();
        let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
        props.power = old_props.power;

        let mut props = Self::get_default_wire_state(world, block_pos, props).await;

        if not_connected && props.is_not_connected() {
            return props;
        }

        let north_connected = props.north.to_wire_connection().is_connected();
        let south_connected = props.south.to_wire_connection().is_connected();
        let east_connected = props.east.to_wire_connection().is_connected();
        let west_connected = props.west.to_wire_connection().is_connected();

        let is_north_south_disconnected = !north_connected && !south_connected;
        let is_east_west_disconnected = !east_connected && !west_connected;

        if !west_connected && is_north_south_disconnected {
            props.west = WestWireConnection::Side;
        }

        if !east_connected && is_north_south_disconnected {
            props.east = EastWireConnection::Side;
        }

        if !north_connected && is_east_west_disconnected {
            props.north = NorthWireConnection::Side;
        }

        if !south_connected && is_east_west_disconnected {
            props.south = SouthWireConnection::Side;
        }

        props
    }
}

trait RedstoneWireLikePropertiesExt {
    fn is_not_connected(&self) -> bool;
    fn is_fully_connected(&self) -> bool;
    fn is_side_connected(&self, direction: BlockDirection) -> bool;
    fn get_connection_type(&self, direction: BlockDirection) -> WireConnection;
}

impl RedstoneWireLikePropertiesExt for RedstoneWireLikeProperties {
    fn is_not_connected(&self) -> bool {
        !self.north.to_wire_connection().is_connected()
            && !self.south.to_wire_connection().is_connected()
            && !self.east.to_wire_connection().is_connected()
            && !self.west.to_wire_connection().is_connected()
    }

    fn is_fully_connected(&self) -> bool {
        self.north.to_wire_connection().is_connected()
            && self.south.to_wire_connection().is_connected()
            && self.east.to_wire_connection().is_connected()
            && self.west.to_wire_connection().is_connected()
    }

    fn is_side_connected(&self, direction: BlockDirection) -> bool {
        match direction {
            BlockDirection::North => self.north.to_wire_connection().is_connected(),
            BlockDirection::South => self.south.to_wire_connection().is_connected(),
            BlockDirection::East => self.east.to_wire_connection().is_connected(),
            BlockDirection::West => self.west.to_wire_connection().is_connected(),
            _ => false,
        }
    }

    fn get_connection_type(&self, direction: BlockDirection) -> WireConnection {
        match direction {
            BlockDirection::North => self.north.to_wire_connection(),
            BlockDirection::South => self.south.to_wire_connection(),
            BlockDirection::East => self.east.to_wire_connection(),
            BlockDirection::West => self.west.to_wire_connection(),
            _ => WireConnection::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WireConnection {
    Up,
    Side,
    None,
}

impl WireConnection {
    fn is_connected(self) -> bool {
        self != Self::None
    }

    fn set_connection(self, props: &mut RedstoneWireLikeProperties, direction: BlockDirection) {
        match direction {
            BlockDirection::North => match self {
                Self::Up => props.north = NorthWireConnection::Up,
                Self::Side => props.north = NorthWireConnection::Side,
                Self::None => props.north = NorthWireConnection::None,
            },
            BlockDirection::South => match self {
                Self::Up => props.south = SouthWireConnection::Up,
                Self::Side => props.south = SouthWireConnection::Side,
                Self::None => props.south = SouthWireConnection::None,
            },
            BlockDirection::East => match self {
                Self::Up => props.east = EastWireConnection::Up,
                Self::Side => props.east = EastWireConnection::Side,
                Self::None => props.east = EastWireConnection::None,
            },
            BlockDirection::West => match self {
                Self::Up => props.west = WestWireConnection::Up,
                Self::Side => props.west = WestWireConnection::Side,
                Self::None => props.west = WestWireConnection::None,
            },
            _ => {}
        }
    }
}

trait CardinalWireConnectionExt {
    fn to_wire_connection(&self) -> WireConnection;
}

impl CardinalWireConnectionExt for NorthWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
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
}

impl CardinalWireConnectionExt for EastWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
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
}
