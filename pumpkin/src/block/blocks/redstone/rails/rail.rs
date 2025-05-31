use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::RailShape;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

use super::StraightRailShapeExt;
use super::common::{can_place_rail_at, rail_placement_is_valid, update_flanking_rails_shape};
use super::{HorizontalFacingRailExt, Rail, RailElevation, RailProperties};

#[pumpkin_block("minecraft:rail")]
pub struct RailBlock;

#[async_trait]
impl PumpkinBlock for RailBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut rail_props = RailProperties::default(block);
        rail_props.set_waterlogged(replacing.water_source());

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
            player
                .living_entity
                .entity
                .get_horizontal_facing()
                .to_rail_shape_flat()
                .as_shape()
        };

        rail_props.set_shape(shape);
        rail_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        update_flanking_rails_shape(world, block, state_id, block_pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        if !rail_placement_is_valid(world, block, block_pos).await {
            world
                .break_block(block_pos, None, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_rail_at(world.unwrap(), block_pos).await
    }
}
