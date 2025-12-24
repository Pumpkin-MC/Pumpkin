use crate::block::BlockFuture;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::EastWallShape;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::NorthWallShape;
use pumpkin_data::block_properties::SouthWallShape;
use pumpkin_data::block_properties::WestWallShape;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::block::BlockBehaviour;
use crate::world::World;
type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;
type FenceLikeProperties = pumpkin_data::block_properties::OakFenceLikeProperties;
type WallProperties = pumpkin_data::block_properties::ResinBrickWallLikeProperties;

#[pumpkin_block_from_tag("minecraft:walls")]
pub struct WallBlock;

impl BlockBehaviour for WallBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut wall_props = WallProperties::default(args.block);
            wall_props.waterlogged = args.replacing.water_source();

            compute_wall_state(wall_props, args.world, args.block, args.position).await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let wall_props = WallProperties::from_state_id(args.state_id, args.block);
            compute_wall_state(wall_props, args.world, args.block, args.position).await
        })
    }
}

pub async fn compute_wall_state(
    mut wall_props: WallProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    let (block_above, block_above_state) = world.get_block_and_state(&block_pos.up()).await;

    for direction in HorizontalFacing::all() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) = world.get_block_and_state(&other_block_pos).await;

        let connected = is_connected(block, direction, other_block, other_block_state);

        let shape = if connected {
            let raise = if block_above_state.is_full_cube() {
                true
            } else if block_above.has_tag(&tag::Block::MINECRAFT_WALLS) {
                let other_props = WallProperties::from_state_id(block_above_state.id, block_above);
                match direction {
                    HorizontalFacing::North => other_props.north != NorthWallShape::None,
                    HorizontalFacing::South => other_props.south != SouthWallShape::None,
                    HorizontalFacing::East => other_props.east != EastWallShape::None,
                    HorizontalFacing::West => other_props.west != WestWallShape::None,
                }
            } else if block_above.has_tag(&tag::Block::C_GLASS_PANES)
                || block_above.has_tag(&tag::Block::MINECRAFT_FENCES)
                || block_above == &Block::IRON_BARS
            {
                let other_props =
                    FenceLikeProperties::from_state_id(block_above_state.id, block_above);
                match direction {
                    HorizontalFacing::North => other_props.north,
                    HorizontalFacing::South => other_props.south,
                    HorizontalFacing::East => other_props.east,
                    HorizontalFacing::West => other_props.west,
                }
            } else if block_above.has_tag(&tag::Block::MINECRAFT_FENCE_GATES) {
                let other_props =
                    FenceGateProperties::from_state_id(block_above_state.id, block_above);
                // gate is perp to connected direction
                let perpendicular_gate = direction == other_props.facing.rotate_clockwise()
                    || direction == other_props.facing.rotate_counter_clockwise();

                perpendicular_gate && !other_props.open
            } else {
                false
            };
            if raise {
                WallShape::Tall
            } else {
                WallShape::Low
            }
        } else {
            WallShape::None
        };

        match direction {
            HorizontalFacing::North => wall_props.north = shape.into(),
            HorizontalFacing::South => wall_props.south = shape.into(),
            HorizontalFacing::East => wall_props.east = shape.into(),
            HorizontalFacing::West => wall_props.west = shape.into(),
        }
    }

    let connected_north_south = wall_props.north != NorthWallShape::None
        && wall_props.south != SouthWallShape::None
        && wall_props.east == EastWallShape::None
        && wall_props.west == WestWallShape::None;
    let connected_east_west = wall_props.north == NorthWallShape::None
        && wall_props.south == SouthWallShape::None
        && wall_props.east != EastWallShape::None
        && wall_props.west != WestWallShape::None;
    let cross = wall_props.north != NorthWallShape::None
        && wall_props.south != SouthWallShape::None
        && wall_props.east != EastWallShape::None
        && wall_props.west != WestWallShape::None;

    wall_props.up = if !(cross || connected_north_south || connected_east_west) {
        true
    } else if block_above.has_tag(&tag::Block::MINECRAFT_WALLS) {
        let other_props = WallProperties::from_state_id(block_above_state.id, block_above);
        other_props.up
    } else if block_above.has_tag(&tag::Block::MINECRAFT_FENCE_GATES) {
        let other_props = FenceGateProperties::from_state_id(block_above_state.id, block_above);
        if other_props.open {
            false
        } else {
            match other_props.facing {
                HorizontalFacing::East | HorizontalFacing::West => connected_east_west,
                HorizontalFacing::South | HorizontalFacing::North => connected_north_south,
            }
        }
    } else {
        false
    };
    wall_props.to_state_id(block)
}

fn is_connected(
    block: &Block,
    direction: HorizontalFacing,
    other_block: &Block,
    other_block_state: &BlockState,
) -> bool {
    let mut connected = other_block == block
        || (other_block_state.is_solid() && other_block_state.is_full_cube())
        || other_block_state.is_side_solid(BlockDirection::from_cardinal_direction(
            direction.opposite(),
        ))
        || other_block.has_tag(&tag::Block::MINECRAFT_WALLS)
        || other_block == &Block::IRON_BARS
        || other_block.has_tag(&tag::Block::C_GLASS_PANES);

    // fence gates do not pass is_side_solid check
    if !connected && other_block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES) {
        let fence_props = FenceGateProperties::from_state_id(other_block_state.id, other_block);
        if fence_props.facing == direction.rotate_clockwise()
            || fence_props.facing == direction.rotate_counter_clockwise()
        {
            connected = true;
        }
    }
    connected
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WallShape {
    None,
    Low,
    Tall,
}

impl From<WallShape> for NorthWallShape {
    fn from(value: WallShape) -> Self {
        match value {
            WallShape::None => Self::None,
            WallShape::Low => Self::Low,
            WallShape::Tall => Self::Tall,
        }
    }
}

impl From<WallShape> for SouthWallShape {
    fn from(value: WallShape) -> Self {
        match value {
            WallShape::None => Self::None,
            WallShape::Low => Self::Low,
            WallShape::Tall => Self::Tall,
        }
    }
}

impl From<WallShape> for EastWallShape {
    fn from(value: WallShape) -> Self {
        match value {
            WallShape::None => Self::None,
            WallShape::Low => Self::Low,
            WallShape::Tall => Self::Tall,
        }
    }
}

impl From<WallShape> for WestWallShape {
    fn from(value: WallShape) -> Self {
        match value {
            WallShape::None => Self::None,
            WallShape::Low => Self::Low,
            WallShape::Tall => Self::Tall,
        }
    }
}
