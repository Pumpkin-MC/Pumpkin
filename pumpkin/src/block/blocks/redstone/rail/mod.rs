use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::RailShape;
use pumpkin_data::block::StraightRailShape;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

use crate::world::World;

pub(crate) mod powered_rail;
pub(crate) mod rail;

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

    async fn find_if_unlocked(
        world: &World,
        place_pos: &BlockPos,
        direction: BlockDirection,
    ) -> Option<Rail> {
        let rail_position = place_pos.offset(direction.to_offset());
        let rail = Rail::find_with_elevation(world, rail_position).await?;

        match rail.is_locked(world).await {
            true => None,
            false => Some(rail),
        }
    }

    async fn is_locked(&self, world: &World) -> bool {
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

    pub fn get_new_rail_shape(&self, first: &BlockDirection, second: &BlockDirection) -> RailShape {
        match (first, second) {
            (BlockDirection::North, BlockDirection::South)
            | (BlockDirection::South, BlockDirection::North) => RailShape::NorthSouth,

            (BlockDirection::East, BlockDirection::West)
            | (BlockDirection::West, BlockDirection::East) => RailShape::EastWest,

            (BlockDirection::South, BlockDirection::East)
            | (BlockDirection::East, BlockDirection::South) => {
                if self.properties.can_curve() {
                    RailShape::SouthEast
                } else {
                    RailShape::EastWest
                }
            }

            (BlockDirection::South, BlockDirection::West)
            | (BlockDirection::West, BlockDirection::South) => {
                if self.properties.can_curve() {
                    RailShape::SouthWest
                } else {
                    RailShape::EastWest
                }
            }

            (BlockDirection::North, BlockDirection::West)
            | (BlockDirection::West, BlockDirection::North) => {
                if self.properties.can_curve() {
                    RailShape::NorthWest
                } else {
                    RailShape::EastWest
                }
            }

            (BlockDirection::North, BlockDirection::East)
            | (BlockDirection::East, BlockDirection::North) => {
                if self.properties.can_curve() {
                    RailShape::NorthEast
                } else {
                    RailShape::EastWest
                }
            }

            _ => unreachable!(
                "Invalid rail direction combination: {:?}, {:?}",
                first, second
            ),
        }
    }
}

enum RailProperties {
    Rail(pumpkin_data::block::RailLikeProperties),
    StraightRail(pumpkin_data::block::PoweredRailLikeProperties),
}

impl RailProperties {
    pub fn default(block: &Block) -> Self {
        if *block == Block::RAIL {
            RailProperties::Rail(pumpkin_data::block::RailLikeProperties::default(block))
        } else {
            RailProperties::StraightRail(pumpkin_data::block::PoweredRailLikeProperties::default(
                block,
            ))
        }
    }

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
                StraightRailShape::NorthSouth => [BlockDirection::South, BlockDirection::North],
                StraightRailShape::EastWest => [BlockDirection::West, BlockDirection::East],
                StraightRailShape::AscendingEast => [BlockDirection::West, BlockDirection::East],
                StraightRailShape::AscendingWest => [BlockDirection::West, BlockDirection::East],
                StraightRailShape::AscendingNorth => [BlockDirection::South, BlockDirection::North],
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

#[derive(Debug, PartialEq)]
enum RailElevation {
    Flat,
    Up,
    Down,
}

async fn compute_neighbor_rail_new_shape(
    world: &World,
    direction: &BlockDirection,
    neighbor_rail: &Rail,
) -> RailShape {
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
            return neighbor_rail.properties.shape();
        }
        1 => [neighbor_connected_torwards[0], direction.opposite()],
        0 => [*direction, direction.opposite()],
        _ => unreachable!("Rails only have two sides"),
    };

    // Handle rails that want to be straight
    if new_neighbor_directions
        .iter()
        .all(|d| d == direction || *d == direction.opposite())
    {
        return if neighbor_rail.elevation == RailElevation::Down {
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
            neighbor_rail
                .get_new_rail_shape(&new_neighbor_directions[0], &new_neighbor_directions[1])
        };
    }

    // Handle straight rails that want to curve
    if !neighbor_rail.properties.can_curve() {
        return if new_neighbor_directions[0] == BlockDirection::North
            || new_neighbor_directions[0] == BlockDirection::South
        {
            if neighbor_rail.elevation == RailElevation::Down {
                // The rail is down so it should be ascending
                match direction {
                    BlockDirection::North => RailShape::AscendingSouth,
                    BlockDirection::South => RailShape::AscendingNorth,
                    BlockDirection::East => RailShape::AscendingWest,
                    BlockDirection::West => RailShape::AscendingEast,
                    _ => unreachable!("Rail cannot point vertically"),
                }
            } else {
                neighbor_rail
                    .get_new_rail_shape(&new_neighbor_directions[0], &new_neighbor_directions[1])
            }
        } else {
            neighbor_rail.properties.shape()
        };
    }

    neighbor_rail.get_new_rail_shape(&new_neighbor_directions[0], &new_neighbor_directions[1])
}
