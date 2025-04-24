use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::RailShape;
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

type RailProperties = pumpkin_data::block::RailLikeProperties;

struct Rail {
    block: Block,
    position: BlockPos,
    properties: RailProperties,
    elevation: RailElevation,
}

impl Rail {
    async fn locked(&self, world: &World) -> bool {
        for direction in self.directions() {
            let Some(other_rail) =
                get_rail_with_elevation(world, self.position.offset(direction.to_offset())).await
            else {
                // Rails pointing to non-rail blocks are not locked
                return false;
            };

            let direction = direction.opposite();
            if !other_rail.directions().into_iter().any(|d| d == direction) {
                // Rails pointing to other rails that are not pointing back are not locked
                return false;
            }
        }

        true
    }

    fn directions(&self) -> [BlockDirection; 2] {
        match self.properties.shape {
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
        }
    }
}

#[derive(Debug, PartialEq)]
enum RailElevation {
    Flat,
    Up,
    Down,
}

fn get_rail_shape(directions: &[BlockDirection]) -> RailShape {
    match directions {
        [BlockDirection::North, BlockDirection::South] => RailShape::NorthSouth,
        [BlockDirection::South, BlockDirection::North] => RailShape::NorthSouth,
        [BlockDirection::East, BlockDirection::West] => RailShape::EastWest,
        [BlockDirection::West, BlockDirection::East] => RailShape::EastWest,
        [BlockDirection::South, BlockDirection::East] => RailShape::SouthEast,
        [BlockDirection::East, BlockDirection::South] => RailShape::SouthEast,
        [BlockDirection::South, BlockDirection::West] => RailShape::SouthWest,
        [BlockDirection::West, BlockDirection::South] => RailShape::SouthWest,
        [BlockDirection::North, BlockDirection::West] => RailShape::NorthWest,
        [BlockDirection::West, BlockDirection::North] => RailShape::NorthWest,
        [BlockDirection::North, BlockDirection::East] => RailShape::NorthEast,
        [BlockDirection::East, BlockDirection::North] => RailShape::NorthEast,
        _ => unreachable!("Invalid rail direction combination: {:?}", directions),
    }
}

async fn get_rail_with_elevation(world: &World, position: BlockPos) -> Option<Rail> {
    let (block, block_state) = world.get_block_and_block_state(&position).await.unwrap();
    if block.is_tagged_with("#minecraft:rails").unwrap() {
        let properties = RailProperties::from_state_id(block_state.id, &block);
        return Some(Rail {
            block,
            position,
            properties,
            elevation: RailElevation::Flat,
        });
    }

    let (block, block_state) = world
        .get_block_and_block_state(&position.up())
        .await
        .unwrap();
    if block.is_tagged_with("#minecraft:rails").unwrap() {
        let properties = RailProperties::from_state_id(block_state.id, &block);
        return Some(Rail {
            block,
            position: position.up(),
            properties,
            elevation: RailElevation::Up,
        });
    }

    let (block, block_state) = world
        .get_block_and_block_state(&position.down())
        .await
        .unwrap();
    if block.is_tagged_with("#minecraft:rails").unwrap() {
        let properties = RailProperties::from_state_id(block_state.id, &block);
        return Some(Rail {
            block,
            position: position.down(),
            properties,
            elevation: RailElevation::Down,
        });
    }

    None
}

async fn get_unlocked_rail(
    world: &World,
    place_pos: &BlockPos,
    direction: BlockDirection,
) -> Option<Rail> {
    let rail_position = place_pos.offset(direction.to_offset());
    let Some(rail) = get_rail_with_elevation(world, rail_position).await else {
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
        let mut rail_props = RailProperties::default(block);

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
        let rail = Rail {
            block: block.clone(),
            position: *block_pos,
            properties: RailProperties::from_state_id(state_id, block),
            elevation: RailElevation::Flat,
        };

        for direction in rail.directions() {
            let Some(mut neighbor_rail) =
                get_rail_with_elevation(world, block_pos.offset(direction.to_offset())).await
            else {
                // Skip non-rail blocks
                continue;
            };

            if neighbor_rail
                .directions()
                .into_iter()
                .all(|d| d == direction || d == direction.opposite())
            {
                // Lazy update straight rails pointing to the placed rail
                if neighbor_rail.elevation == RailElevation::Down {
                    neighbor_rail.properties.shape = match direction {
                        BlockDirection::North => RailShape::AscendingSouth,
                        BlockDirection::South => {
                            if neighbor_rail.properties.shape == RailShape::NorthSouth {
                                RailShape::AscendingNorth
                            } else {
                                RailShape::AscendingSouth
                            }
                        }
                        BlockDirection::East => RailShape::AscendingWest,
                        BlockDirection::West => {
                            if neighbor_rail.properties.shape == RailShape::EastWest {
                                RailShape::AscendingEast
                            } else {
                                RailShape::AscendingWest
                            }
                        }
                        _ => unreachable!("Rail cannot point vertically"),
                    }
                }

                world
                    .set_block_state(
                        &neighbor_rail.position,
                        neighbor_rail.properties.to_state_id(&neighbor_rail.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                continue;
            }

            let mut connections = Vec::with_capacity(2);
            for other_direction in neighbor_rail.directions() {
                if other_direction == direction.opposite() {
                    // Rails pointing to where the player placed are connected
                    connections.push(other_direction);
                    continue;
                }

                let Some(maybe_connected_rail) = get_rail_with_elevation(
                    world,
                    neighbor_rail.position.offset(other_direction.to_offset()),
                )
                .await
                else {
                    // Rails pointing to non-rail blocks are not connected
                    continue;
                };

                if !maybe_connected_rail
                    .directions()
                    .into_iter()
                    .any(|d| d == other_direction.opposite())
                {
                    // Rails pointing to other rails that are not pointing back are not connected
                    continue;
                }

                connections.push(other_direction);
            }

            match connections.len() {
                2 => {
                    // Skip locked rails
                    continue;
                }
                1 => {
                    if connections.first().unwrap() != &direction.opposite() {
                        connections.push(direction.opposite());
                    } else {
                        connections.push(direction);
                    }
                }
                0 => {
                    connections.push(direction);
                    connections.push(direction.opposite());
                }
                _ => unreachable!("Rails only have two sides"),
            }

            neighbor_rail.properties.shape = get_rail_shape(&connections);
            if neighbor_rail.elevation == RailElevation::Down {
                neighbor_rail.properties.shape = match direction {
                    BlockDirection::North => RailShape::AscendingSouth,
                    BlockDirection::South => RailShape::AscendingNorth,
                    BlockDirection::West => RailShape::AscendingEast,
                    BlockDirection::East => RailShape::AscendingWest,
                    _ => unreachable!("Rail cannot point vertically"),
                }
            }

            world
                .set_block_state(
                    &neighbor_rail.position,
                    neighbor_rail.properties.to_state_id(&neighbor_rail.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn can_place_at(&self, world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(&pos.down()).await.unwrap();
        state.is_solid
    }
}
