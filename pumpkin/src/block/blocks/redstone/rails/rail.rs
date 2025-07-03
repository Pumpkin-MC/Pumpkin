use async_trait::async_trait;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::RailShape;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::OnNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::PumpkinBlock;

use super::StraightRailShapeExt;
use super::common::{can_place_rail_at, rail_placement_is_valid, update_flanking_rails_shape};
use super::{HorizontalFacingRailExt, Rail, RailElevation, RailProperties};

#[pumpkin_block("minecraft:rail")]
pub struct RailBlock;

#[async_trait]
impl PumpkinBlock for RailBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let world = args.world;
        let block_pos = args.location;
        let mut rail_props = RailProperties::default(args.block);
        rail_props.set_waterlogged(args.replacing.water_source());

        let shape = if let Some(east_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::East).await
        {
            if Rail::find_if_unlocked(world, block_pos, HorizontalFacing::South)
                .await
                .is_some()
            {
                RailShape::SouthEast
            } else if Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North)
                .await
                .is_some()
            {
                RailShape::NorthEast
            } else {
                match Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West).await {
                    Some(west_rail) if west_rail.elevation == RailElevation::Up => {
                        RailShape::AscendingWest
                    }
                    _ => {
                        if east_rail.elevation == RailElevation::Up {
                            RailShape::AscendingEast
                        } else {
                            RailShape::EastWest
                        }
                    }
                }
            }
        } else if let Some(south_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::South).await
        {
            if Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West)
                .await
                .is_some()
            {
                RailShape::SouthWest
            } else if south_rail.elevation == RailElevation::Up {
                RailShape::AscendingSouth
            } else {
                match Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North).await {
                    Some(north_rail) if north_rail.elevation == RailElevation::Up => {
                        RailShape::AscendingNorth
                    }
                    _ => RailShape::NorthSouth,
                }
            }
        } else if let Some(west_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West).await
        {
            if Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North)
                .await
                .is_some()
            {
                RailShape::NorthWest
            } else if west_rail.elevation == RailElevation::Up {
                RailShape::AscendingWest
            } else {
                RailShape::EastWest
            }
        } else if let Some(north_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North).await
        {
            if north_rail.elevation == RailElevation::Up {
                RailShape::AscendingNorth
            } else {
                RailShape::NorthSouth
            }
        } else {
            args.player
                .living_entity
                .entity
                .get_horizontal_facing()
                .to_rail_shape_flat()
                .as_shape()
        };

        rail_props.set_shape(shape);
        rail_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        update_flanking_rails_shape(args.world, args.block, args.state_id, args.location).await;
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.location).await {
            args.world
                .break_block(args.location, None, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.location).await
    }
}
