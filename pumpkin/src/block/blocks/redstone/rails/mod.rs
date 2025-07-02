use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::PoweredRailLikeProperties;
use pumpkin_data::block_properties::RailLikeProperties;
use pumpkin_data::block_properties::RailShape;
use pumpkin_data::block_properties::StraightRailShape;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::world::World;

mod common;

pub mod activator_rail;
pub mod detector_rail;
pub mod powered_rail;
pub mod rail;

struct Rail {
    block: &'static Block,
    position: BlockPos,
    properties: RailProperties,
    elevation: RailElevation,
}

impl Rail {
    async fn find_with_elevation(world: &World, position: BlockPos) -> Option<Self> {
        let (block, block_state) = world.get_block_and_block_state(&position).await;
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, block);
            return Some(Self {
                block,
                position,
                properties,
                elevation: RailElevation::Flat,
            });
        }

        let pos = position.up();
        let (block, block_state) = world.get_block_and_block_state(&pos).await;
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, block);
            return Some(Self {
                block,
                position: pos,
                properties,
                elevation: RailElevation::Up,
            });
        }

        let pos = position.down();
        let (block, block_state) = world.get_block_and_block_state(&pos).await;
        if block.is_tagged_with("#minecraft:rails").unwrap() {
            let properties = RailProperties::new(block_state.id, block);
            return Some(Self {
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
        direction: HorizontalFacing,
    ) -> Option<Self> {
        let rail_position = place_pos.offset(direction.to_offset());
        let rail = Self::find_with_elevation(world, rail_position).await?;

        if rail.is_locked(world).await {
            return None;
        }

        Some(rail)
    }

    async fn is_locked(&self, world: &World) -> bool {
        for direction in self.properties.directions() {
            let Some(other_rail) =
                Self::find_with_elevation(world, self.position.offset(direction.to_offset())).await
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

    pub fn get_new_rail_shape(
        &self,
        first: HorizontalFacing,
        second: HorizontalFacing,
    ) -> RailShape {
        match (first, second) {
            (HorizontalFacing::North, HorizontalFacing::South)
            | (HorizontalFacing::South, HorizontalFacing::North) => RailShape::NorthSouth,

            (HorizontalFacing::East, HorizontalFacing::West)
            | (HorizontalFacing::West, HorizontalFacing::East) => RailShape::EastWest,

            (HorizontalFacing::South, HorizontalFacing::East)
            | (HorizontalFacing::East, HorizontalFacing::South) => {
                if self.properties.can_curve() {
                    RailShape::SouthEast
                } else {
                    RailShape::EastWest
                }
            }

            (HorizontalFacing::South, HorizontalFacing::West)
            | (HorizontalFacing::West, HorizontalFacing::South) => {
                if self.properties.can_curve() {
                    RailShape::SouthWest
                } else {
                    RailShape::EastWest
                }
            }

            (HorizontalFacing::North, HorizontalFacing::West)
            | (HorizontalFacing::West, HorizontalFacing::North) => {
                if self.properties.can_curve() {
                    RailShape::NorthWest
                } else {
                    RailShape::EastWest
                }
            }

            (HorizontalFacing::North, HorizontalFacing::East)
            | (HorizontalFacing::East, HorizontalFacing::North) => {
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
    Rail(RailLikeProperties),
    StraightRail(PoweredRailLikeProperties),
}

impl RailProperties {
    pub fn default(block: &Block) -> Self {
        if block == &Block::RAIL {
            Self::Rail(RailLikeProperties::default(block))
        } else {
            Self::StraightRail(PoweredRailLikeProperties::default(block))
        }
    }

    pub fn new(state_id: u16, block: &Block) -> Self {
        if block == &Block::RAIL {
            Self::Rail(RailLikeProperties::from_state_id(state_id, block))
        } else {
            Self::StraightRail(PoweredRailLikeProperties::from_state_id(state_id, block))
        }
    }

    fn can_curve(&self) -> bool {
        match self {
            Self::Rail(_) => true,
            Self::StraightRail(_) => false,
        }
    }

    fn shape(&self) -> RailShape {
        match self {
            Self::Rail(props) => props.shape,
            Self::StraightRail(props) => match props.shape {
                StraightRailShape::NorthSouth => RailShape::NorthSouth,
                StraightRailShape::EastWest => RailShape::EastWest,
                StraightRailShape::AscendingEast => RailShape::AscendingEast,
                StraightRailShape::AscendingWest => RailShape::AscendingWest,
                StraightRailShape::AscendingNorth => RailShape::AscendingNorth,
                StraightRailShape::AscendingSouth => RailShape::AscendingSouth,
            },
        }
    }

    fn directions(&self) -> [HorizontalFacing; 2] {
        match self {
            Self::Rail(props) => match props.shape {
                RailShape::EastWest | RailShape::AscendingEast | RailShape::AscendingWest => {
                    [HorizontalFacing::West, HorizontalFacing::East]
                }
                RailShape::NorthSouth | RailShape::AscendingNorth | RailShape::AscendingSouth => {
                    [HorizontalFacing::North, HorizontalFacing::South]
                }
                RailShape::SouthEast => [HorizontalFacing::East, HorizontalFacing::South],
                RailShape::SouthWest => [HorizontalFacing::West, HorizontalFacing::South],
                RailShape::NorthWest => [HorizontalFacing::West, HorizontalFacing::North],
                RailShape::NorthEast => [HorizontalFacing::East, HorizontalFacing::North],
            },

            Self::StraightRail(props) => match props.shape {
                StraightRailShape::EastWest
                | StraightRailShape::AscendingEast
                | StraightRailShape::AscendingWest => {
                    [HorizontalFacing::West, HorizontalFacing::East]
                }

                StraightRailShape::NorthSouth
                | StraightRailShape::AscendingNorth
                | StraightRailShape::AscendingSouth => {
                    [HorizontalFacing::North, HorizontalFacing::South]
                }
            },
        }
    }

    fn to_state_id(&self, block: &Block) -> BlockStateId {
        match self {
            Self::Rail(props) => props.to_state_id(block),
            Self::StraightRail(props) => props.to_state_id(block),
        }
    }

    fn set_waterlogged(&mut self, waterlogged: bool) {
        match self {
            Self::Rail(props) => props.waterlogged = waterlogged,
            Self::StraightRail(props) => props.waterlogged = waterlogged,
        }
    }

    fn set_shape(&mut self, shape: RailShape) {
        match self {
            Self::Rail(props) => props.shape = shape,
            Self::StraightRail(props) => {
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

    fn set_straight_shape(&mut self, shape: StraightRailShape) {
        match self {
            Self::Rail(props) => props.shape = shape.as_shape(),
            Self::StraightRail(props) => props.shape = shape,
        }
    }
}

#[derive(Debug, PartialEq)]
enum RailElevation {
    Flat,
    Up,
    Down,
}

pub trait StraightRailShapeExt {
    fn as_shape(&self) -> RailShape;
}

impl StraightRailShapeExt for StraightRailShape {
    fn as_shape(&self) -> RailShape {
        match self {
            Self::NorthSouth => RailShape::NorthSouth,
            Self::EastWest => RailShape::EastWest,
            Self::AscendingNorth => RailShape::AscendingNorth,
            Self::AscendingSouth => RailShape::AscendingSouth,
            Self::AscendingEast => RailShape::AscendingEast,
            Self::AscendingWest => RailShape::AscendingWest,
        }
    }
}

pub trait HorizontalFacingRailExt {
    fn to_rail_shape_flat(&self) -> StraightRailShape;
    fn to_rail_shape_ascending_towards(&self) -> StraightRailShape;
}

impl HorizontalFacingRailExt for HorizontalFacing {
    fn to_rail_shape_flat(&self) -> StraightRailShape {
        match self {
            Self::North | Self::South => StraightRailShape::NorthSouth,
            Self::East | Self::West => StraightRailShape::EastWest,
        }
    }

    fn to_rail_shape_ascending_towards(&self) -> StraightRailShape {
        match self {
            Self::North => StraightRailShape::AscendingNorth,
            Self::South => StraightRailShape::AscendingSouth,
            Self::East => StraightRailShape::AscendingEast,
            Self::West => StraightRailShape::AscendingWest,
        }
    }
}
