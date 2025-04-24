use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::RailLikeProperties;
use pumpkin_data::block::RailShape;
use pumpkin_data::block::StraightRailShape;
use pumpkin_data::tag::Tagable;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::world::BlockFlags;
use crate::world::World;
use crate::{entity::player::Player, server::Server};

enum RailProperties {
    Rail(pumpkin_data::block::RailLikeProperties),
    StraightRail(pumpkin_data::block::PoweredRailLikeProperties),
}

impl RailProperties {
    pub fn new(state_id: u16, block: &Block) -> Self {
        if *block == Block::RAIL {
            RailProperties::Rail(pumpkin_data::block::RailLikeProperties::from_state_id(
                state_id, block,
            ))
        } else {
            RailProperties::StraightRail(
                pumpkin_data::block::PoweredRailLikeProperties::from_state_id(state_id, block),
            )
        }
    }

    fn can_curve(&self) -> bool {
        match self {
            RailProperties::Rail(_) => true,
            RailProperties::StraightRail(_) => false,
        }
    }

    fn shape(&self) -> RailShape {
        match self {
            RailProperties::Rail(props) => props.shape,
            RailProperties::StraightRail(props) => match props.shape {
                StraightRailShape::NorthSouth => RailShape::NorthSouth,
                StraightRailShape::EastWest => RailShape::EastWest,
                StraightRailShape::AscendingEast => RailShape::AscendingEast,
                StraightRailShape::AscendingWest => RailShape::AscendingWest,
                StraightRailShape::AscendingNorth => RailShape::AscendingNorth,
                StraightRailShape::AscendingSouth => RailShape::AscendingSouth,
            },
        }
    }

    fn directions(&self) -> [BlockDirection; 2] {
        match self {
            RailProperties::Rail(props) => match props.shape {
                RailShape::NorthSouth => [BlockDirection::North, BlockDirection::South],
                RailShape::EastWest => [BlockDirection::East, BlockDirection::West],
                RailShape::AscendingEast => [BlockDirection::East, BlockDirection::West],
                RailShape::AscendingWest => [BlockDirection::West, BlockDirection::East],
                RailShape::AscendingNorth => [BlockDirection::North, BlockDirection::South],
                RailShape::AscendingSouth => [BlockDirection::South, BlockDirection::North],
                RailShape::SouthEast => [BlockDirection::South, BlockDirection::East],
                RailShape::SouthWest => [BlockDirection::South, BlockDirection::West],
                RailShape::NorthWest => [BlockDirection::North, BlockDirection::West],
                RailShape::NorthEast => [BlockDirection::North, BlockDirection::East],
            },
            RailProperties::StraightRail(props) => match props.shape {
                StraightRailShape::NorthSouth => [BlockDirection::North, BlockDirection::South],
                StraightRailShape::EastWest => [BlockDirection::East, BlockDirection::West],
                StraightRailShape::AscendingEast => [BlockDirection::East, BlockDirection::West],
                StraightRailShape::AscendingWest => [BlockDirection::West, BlockDirection::East],
                StraightRailShape::AscendingNorth => [BlockDirection::North, BlockDirection::South],
                StraightRailShape::AscendingSouth => [BlockDirection::South, BlockDirection::North],
            },
        }
    }

    fn to_state_id(&self, block: &Block) -> BlockStateId {
        match self {
            RailProperties::Rail(props) => props.to_state_id(block),
            RailProperties::StraightRail(props) => props.to_state_id(block),
        }
    }

    fn set_shape(&mut self, shape: RailShape) {
        match self {
            RailProperties::Rail(props) => props.shape = shape,
            RailProperties::StraightRail(props) => {
                props.shape = match shape {
                    RailShape::NorthSouth => StraightRailShape::NorthSouth,
                    RailShape::EastWest => StraightRailShape::EastWest,
                    RailShape::AscendingEast => StraightRailShape::AscendingEast,
                    RailShape::AscendingWest => StraightRailShape::AscendingWest,
                    RailShape::AscendingNorth => StraightRailShape::AscendingNorth,
                    RailShape::AscendingSouth => StraightRailShape::AscendingSouth,
                    _ => unreachable!("Trying to make a straight rail curved: {:?}", shape),
                }
            }
        }
    }
}

struct Rail {
    block: Block,
    position: BlockPos,
    properties: RailProperties,
    elevation: RailElevation,
}

impl Rail {
    async fn find_with_elevation(world: &World, position: BlockPos) -> Option<Rail> {
        let (block, block_state) = world.get_block_and_block_state(&position).await.unwrap();
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, &block);
            return Some(Rail {
                block,
                position,
                properties,
                elevation: RailElevation::Flat,
            });
        }

        let pos = position.up();
        let (block, block_state) = world.get_block_and_block_state(&pos).await.unwrap();
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, &block);
            return Some(Rail {
                block,
                position: pos,
                properties,
                elevation: RailElevation::Up,
            });
        }

        let pos = position.down();
        let (block, block_state) = world.get_block_and_block_state(&pos).await.unwrap();
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, &block);
            return Some(Rail {
                block,
                position: pos,
                properties,
                elevation: RailElevation::Down,
            });
        }

        None
    }

    async fn locked(&self, world: &World) -> bool {
        for direction in self.properties.directions() {
            let Some(other_rail) =
                Rail::find_with_elevation(world, self.position.offset(direction.to_offset())).await
            else {
                // Rails pointing to non-rail blocks are not locked
                return false;
            };

            let direction = direction.opposite();
            if !other_rail
                .properties
                .directions()
                .into_iter()
                .any(|d| d == direction)
            {
                // Rails pointing to other rails that are not pointing back are not locked
                return false;
            }
        }

        true
    }
}

#[derive(Debug, PartialEq)]
enum RailElevation {
    Flat,
    Up,
    Down,
}

fn get_rail_shape(first: &BlockDirection, second: &BlockDirection) -> RailShape {
    match (first, second) {
        (BlockDirection::North, BlockDirection::South) => RailShape::NorthSouth,
        (BlockDirection::South, BlockDirection::North) => RailShape::NorthSouth,
        (BlockDirection::East, BlockDirection::West) => RailShape::EastWest,
        (BlockDirection::West, BlockDirection::East) => RailShape::EastWest,
        (BlockDirection::South, BlockDirection::East) => RailShape::SouthEast,
        (BlockDirection::East, BlockDirection::South) => RailShape::SouthEast,
        (BlockDirection::South, BlockDirection::West) => RailShape::SouthWest,
        (BlockDirection::West, BlockDirection::South) => RailShape::SouthWest,
        (BlockDirection::North, BlockDirection::West) => RailShape::NorthWest,
        (BlockDirection::West, BlockDirection::North) => RailShape::NorthWest,
        (BlockDirection::North, BlockDirection::East) => RailShape::NorthEast,
        (BlockDirection::East, BlockDirection::North) => RailShape::NorthEast,
        _ => unreachable!(
            "Invalid rail direction combination: {:?}, {:?}",
            first, second
        ),
    }
}

async fn get_unlocked_rail(
    world: &World,
    place_pos: &BlockPos,
    direction: BlockDirection,
) -> Option<Rail> {
    let rail_position = place_pos.offset(direction.to_offset());
    let Some(rail) = Rail::find_with_elevation(world, rail_position).await else {
        return None;
    };

    match rail.locked(world).await {
        true => None,
        false => Some(rail),
    }
}

#[pumpkin_block("minecraft:rail")]
pub struct RailBlock;

#[async_trait]
impl PumpkinBlock for RailBlock {
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
        let mut rail_props = RailLikeProperties::default(block);

        rail_props.shape = if let Some(east_rail) =
            get_unlocked_rail(world, block_pos, BlockDirection::East).await
        {
            if get_unlocked_rail(world, block_pos, BlockDirection::South)
                .await
                .is_some()
            {
                RailShape::SouthEast
            } else if get_unlocked_rail(world, block_pos, BlockDirection::North)
                .await
                .is_some()
            {
                RailShape::NorthEast
            } else {
                match get_unlocked_rail(world, block_pos, BlockDirection::West).await {
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
            get_unlocked_rail(world, block_pos, BlockDirection::South).await
        {
            if get_unlocked_rail(world, block_pos, BlockDirection::West)
                .await
                .is_some()
            {
                RailShape::SouthWest
            } else {
                if south_rail.elevation == RailElevation::Up {
                    RailShape::AscendingSouth
                } else {
                    match get_unlocked_rail(world, block_pos, BlockDirection::North).await {
                        Some(north_rail) if north_rail.elevation == RailElevation::Up => {
                            RailShape::AscendingNorth
                        }
                        _ => RailShape::NorthSouth,
                    }
                }
            }
        } else if let Some(west_rail) =
            get_unlocked_rail(world, block_pos, BlockDirection::West).await
        {
            if get_unlocked_rail(world, block_pos, BlockDirection::North)
                .await
                .is_some()
            {
                RailShape::NorthWest
            } else {
                if west_rail.elevation == RailElevation::Up {
                    RailShape::AscendingWest
                } else {
                    RailShape::EastWest
                }
            }
        } else if let Some(north_rail) =
            get_unlocked_rail(world, block_pos, BlockDirection::North).await
        {
            if north_rail.elevation == RailElevation::Up {
                RailShape::AscendingNorth
            } else {
                RailShape::NorthSouth
            }
        } else {
            match player.living_entity.entity.get_horizontal_facing() {
                HorizontalFacing::North => RailShape::NorthSouth,
                HorizontalFacing::South => RailShape::NorthSouth,
                HorizontalFacing::West => RailShape::EastWest,
                HorizontalFacing::East => RailShape::EastWest,
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
