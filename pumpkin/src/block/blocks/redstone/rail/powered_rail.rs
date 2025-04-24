use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::PoweredRailLikeProperties;
use pumpkin_data::block::RailShape;
use pumpkin_data::block::StraightRailShape;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

use super::{Rail, RailElevation, RailProperties, get_rail_shape};

#[pumpkin_block("minecraft:rail")]
pub struct PoweredRailBlock;

#[async_trait]
impl PumpkinBlock for PoweredRailBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player: &Player,
        _other: bool,
    ) -> BlockStateId {
        let mut rail_props = PoweredRailLikeProperties::default(block);

        rail_props.shape = if let Some(east_rail) =
            Rail::find_if_unlocked(world, block_pos, BlockDirection::East).await
        {
            if Rail::find_if_unlocked(world, block_pos, BlockDirection::South)
                .await
                .is_some()
            {
                StraightRailShape::SouthEast
            } else if Rail::find_if_unlocked(world, block_pos, BlockDirection::North)
                .await
                .is_some()
            {
                StraightRailShape::NorthEast
            } else {
                match Rail::find_if_unlocked(world, block_pos, BlockDirection::West).await {
                    Some(west_rail) if west_rail.elevation == RailElevation::Up => {
                        StraightRailShape::AscendingWest
                    }
                    _ => {
                        if east_rail.elevation == RailElevation::Up {
                            StraightRailShape::AscendingEast
                        } else {
                            StraightRailShape::EastWest
                        }
                    }
                }
            }
        } else if let Some(south_rail) =
            Rail::find_if_unlocked(world, block_pos, BlockDirection::South).await
        {
            if Rail::find_if_unlocked(world, block_pos, BlockDirection::West)
                .await
                .is_some()
            {
                StraightRailShape::SouthWest
            } else {
                if south_rail.elevation == RailElevation::Up {
                    StraightRailShape::AscendingSouth
                } else {
                    match Rail::find_if_unlocked(world, block_pos, BlockDirection::North).await {
                        Some(north_rail) if north_rail.elevation == RailElevation::Up => {
                            StraightRailShape::AscendingNorth
                        }
                        _ => StraightRailShape::NorthSouth,
                    }
                }
            }
        } else if let Some(west_rail) =
            Rail::find_if_unlocked(world, block_pos, BlockDirection::West).await
        {
            if Rail::find_if_unlocked(world, block_pos, BlockDirection::North)
                .await
                .is_some()
            {
                StraightRailShape::NorthWest
            } else {
                if west_rail.elevation == RailElevation::Up {
                    StraightRailShape::AscendingWest
                } else {
                    StraightRailShape::EastWest
                }
            }
        } else if let Some(north_rail) =
            Rail::find_if_unlocked(world, block_pos, BlockDirection::North).await
        {
            if north_rail.elevation == RailElevation::Up {
                StraightRailShape::AscendingNorth
            } else {
                StraightRailShape::NorthSouth
            }
        } else {
            match player.living_entity.entity.get_horizontal_facing() {
                HorizontalFacing::North => StraightRailShape::NorthSouth,
                HorizontalFacing::South => StraightRailShape::NorthSouth,
                HorizontalFacing::West => StraightRailShape::EastWest,
                HorizontalFacing::East => StraightRailShape::EastWest,
            }
        };

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
        for direction in RailProperties::new(state_id, block).directions() {
            let Some(mut neighbor_rail) =
                Rail::find_with_elevation(world, block_pos.offset(direction.to_offset())).await
            else {
                // Skip non-rail blocks
                continue;
            };

            let mut neighbor_connected_torwards = Vec::with_capacity(2);
            let mut neighbor_already_connected_to_elevated = false;
            for neighbor_direction in neighbor_rail.properties.directions() {
                if neighbor_direction == direction.opposite() {
                    // Rails pointing to where the player placed are not connected
                    continue;
                }

                let Some(maybe_connected_rail) = Rail::find_with_elevation(
                    world,
                    neighbor_rail
                        .position
                        .offset(neighbor_direction.to_offset()),
                )
                .await
                else {
                    // Rails pointing to non-rail blocks are not connected
                    continue;
                };

                if maybe_connected_rail
                    .properties
                    .directions()
                    .into_iter()
                    .any(|d| d == neighbor_direction.opposite())
                {
                    // Rails pointing to other rails that are pointing back are connected
                    neighbor_connected_torwards.push(neighbor_direction);
                    neighbor_already_connected_to_elevated =
                        maybe_connected_rail.elevation == RailElevation::Up;
                }
            }

            let new_neighbor_directions = match neighbor_connected_torwards.len() {
                2 => {
                    // Do not update rails that are locked (aka fully connected)
                    continue;
                }
                1 => [neighbor_connected_torwards[0], direction.opposite()],
                0 => [direction, direction.opposite()],
                _ => unreachable!("Rails only have two sides"),
            };

            // Get the final rail shape
            let new_shape = if new_neighbor_directions
                .iter()
                .all(|d| d == &direction || d == &direction.opposite())
            {
                if neighbor_rail.elevation == RailElevation::Down {
                    // The rail is down so it should be ascending
                    match direction {
                        BlockDirection::North => RailShape::AscendingSouth,
                        BlockDirection::South => {
                            if neighbor_already_connected_to_elevated {
                                RailShape::AscendingSouth
                            } else {
                                RailShape::AscendingNorth
                            }
                        }
                        BlockDirection::East => RailShape::AscendingWest,
                        BlockDirection::West => {
                            if neighbor_already_connected_to_elevated {
                                RailShape::AscendingWest
                            } else {
                                RailShape::AscendingEast
                            }
                        }
                        _ => unreachable!("Rail cannot point vertically"),
                    }
                } else if neighbor_already_connected_to_elevated {
                    match neighbor_connected_torwards[0] {
                        BlockDirection::North => RailShape::AscendingNorth,
                        BlockDirection::South => RailShape::AscendingSouth,
                        BlockDirection::East => RailShape::AscendingEast,
                        BlockDirection::West => RailShape::AscendingWest,
                        _ => unreachable!("Rail cannot point vertically"),
                    }
                } else {
                    // Reset the shape to flat even if the rail already had good directions
                    get_rail_shape(&new_neighbor_directions[0], &new_neighbor_directions[1])
                }
            } else {
                get_rail_shape(&new_neighbor_directions[0], &new_neighbor_directions[1])
            };

            if new_shape != neighbor_rail.properties.shape() {
                neighbor_rail.properties.set_shape(new_shape);
                world
                    .set_block_state(
                        &neighbor_rail.position,
                        neighbor_rail.properties.to_state_id(&neighbor_rail.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
    }

    async fn can_place_at(&self, world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(&pos.down()).await.unwrap();
        state.is_solid
    }
}
