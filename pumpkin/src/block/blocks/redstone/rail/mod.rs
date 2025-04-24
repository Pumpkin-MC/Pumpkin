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

pub(self) struct Rail {
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
        let Some(rail) = Rail::find_with_elevation(world, rail_position).await else {
            return None;
        };
    
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
}

pub(self) enum RailProperties {
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

#[derive(Debug, PartialEq)]
pub(self) enum RailElevation {
    Flat,
    Up,
    Down,
}

pub fn get_rail_shape(first: &BlockDirection, second: &BlockDirection) -> RailShape {
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
