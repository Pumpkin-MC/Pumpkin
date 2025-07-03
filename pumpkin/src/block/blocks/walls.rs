use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::EastWallShape;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::NorthWallShape;
use pumpkin_data::block_properties::OakTrapdoorLikeProperties;
use pumpkin_data::block_properties::SouthWallShape;
use pumpkin_data::block_properties::WestWallShape;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

type WallProperties = pumpkin_data::block_properties::ResinBrickWallLikeProperties;
type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;
type FenceLikeProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;

pub struct WallBlock;
impl BlockMetadata for WallBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:walls").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for WallBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut wall_props = WallProperties::default(args.block);
        wall_props.waterlogged = args.replacing.water_source();

        compute_wall_state(wall_props, args.world, args.block, args.location).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let wall_props = WallProperties::from_state_id(args.state_id, args.block);
        compute_wall_state(wall_props, args.world, args.block, args.location).await
    }
}

// TODO: when a piston base is next to a wall with a solid block on top then EXTENDS or RETRACTS the STATE of the connected wall WILL change
pub async fn compute_wall_state(
    mut wall_props: WallProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    let (block_above, block_above_state) = world.get_block_and_block_state(&block_pos.up()).await;

    for direction in HorizontalFacing::all() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) =
            world.get_block_and_block_state(&other_block_pos).await;

        let mut connected = other_block == block
            || (other_block_state.is_solid() && other_block_state.is_full_cube())
            || other_block.is_tagged_with("minecraft:walls").unwrap()
            || other_block == &Block::IRON_BARS
            || other_block.is_tagged_with("c:glass_panes").unwrap();

        if !connected && other_block.is_tagged_with("minecraft:trapdoors").unwrap() {
            let trapdoor_props =
                OakTrapdoorLikeProperties::from_state_id(other_block_state.id, other_block);
            if trapdoor_props.open && trapdoor_props.facing == direction {
                log::info!("Connected = true");
                connected = true;
            }
        }

        if !connected && other_block.is_tagged_with("minecraft:fence_gates").unwrap() {
            let fence_props = FenceGateProperties::from_state_id(other_block_state.id, other_block);
            if fence_props.facing == direction.rotate_clockwise()
                || fence_props.facing == direction.rotate_counter_clockwise()
            {
                connected = true
            }
        }

        let shape = if connected {
            let raise = if block_above_state.is_full_cube() {
                true
            } else if block_above.is_tagged_with("minecraft:walls").unwrap() {
                let other_props = WallProperties::from_state_id(block_above_state.id, block_above);
                match direction {
                    HorizontalFacing::North => other_props.north != NorthWallShape::None,
                    HorizontalFacing::South => other_props.south != SouthWallShape::None,
                    HorizontalFacing::East => other_props.east != EastWallShape::None,
                    HorizontalFacing::West => other_props.west != WestWallShape::None,
                }
            }
            // Match FENCELIKE above and steal shape
            else if block_above.is_tagged_with("c:glass_panes").unwrap()
                || block_above.is_tagged_with("minecraft:fences").unwrap()
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
            } else if block_above.is_tagged_with("minecraft:fence_gates").unwrap() {
                let other_props =
                    FenceGateProperties::from_state_id(block_above_state.id, block_above);

                direction == other_props.facing.rotate_clockwise()
                    || direction == other_props.facing.rotate_counter_clockwise()
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

    let line_north_south = wall_props.north != NorthWallShape::None
        && wall_props.south != SouthWallShape::None
        && wall_props.east == EastWallShape::None
        && wall_props.west == WestWallShape::None;
    let line_east_west = wall_props.north == NorthWallShape::None
        && wall_props.south == SouthWallShape::None
        && wall_props.east != EastWallShape::None
        && wall_props.west != WestWallShape::None;
    let cross = wall_props.north != NorthWallShape::None
        && wall_props.south != SouthWallShape::None
        && wall_props.east != EastWallShape::None
        && wall_props.west != WestWallShape::None;

    wall_props.up = if !(cross || line_north_south || line_east_west) {
        true
    } else if block_above.is_tagged_with("minecraft:walls").unwrap() {
        let other_props = WallProperties::from_state_id(block_above_state.id, block_above);
        other_props.up
    } else {
        false
    };

    // some unique cases if the block above is a fence gate
    if block_above.is_tagged_with("minecraft:fence_gates").unwrap() {
        let above_fence_props =
            FenceGateProperties::from_state_id(block_above_state.id, block_above);
        if above_fence_props.open {
            if line_east_west
                && (above_fence_props.facing == HorizontalFacing::South
                    || above_fence_props.facing == HorizontalFacing::North)
            {
                wall_props.up = false;
                wall_props.east = EastWallShape::from(WallShape::Low);
                wall_props.west = WestWallShape::from(WallShape::Low);
            } else if line_north_south
                && (above_fence_props.facing == HorizontalFacing::East
                    || above_fence_props.facing == HorizontalFacing::West)
            {
                wall_props.up = false;
                wall_props.south = SouthWallShape::from(WallShape::Low);
                wall_props.north = NorthWallShape::from(WallShape::Low);
            } else {
                wall_props.up = false;
            }
        }
    }

    wall_props.to_state_id(block)
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
