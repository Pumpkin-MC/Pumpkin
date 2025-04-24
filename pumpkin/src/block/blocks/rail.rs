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

fn get_rail_directions(shape: RailShape) -> [BlockDirection; 2] {
    match shape {
        RailShape::NorthSouth => [BlockDirection::North, BlockDirection::South],
        RailShape::EastWest => [BlockDirection::East, BlockDirection::West],
        RailShape::AscendingEast => [BlockDirection::East, BlockDirection::Up],
        RailShape::AscendingWest => [BlockDirection::West, BlockDirection::Up],
        RailShape::AscendingNorth => [BlockDirection::North, BlockDirection::Up],
        RailShape::AscendingSouth => [BlockDirection::South, BlockDirection::Up],
        RailShape::SouthEast => [BlockDirection::South, BlockDirection::East],
        RailShape::SouthWest => [BlockDirection::South, BlockDirection::West],
        RailShape::NorthWest => [BlockDirection::North, BlockDirection::West],
        RailShape::NorthEast => [BlockDirection::North, BlockDirection::East],
    }
}

fn get_rail_shape(directions: &[BlockDirection]) -> RailShape {
    match directions {
        [BlockDirection::North, BlockDirection::South] => RailShape::NorthSouth,
        [BlockDirection::South, BlockDirection::North] => RailShape::NorthSouth,
        [BlockDirection::East, BlockDirection::West] => RailShape::EastWest,
        [BlockDirection::West, BlockDirection::East] => RailShape::EastWest,
        [BlockDirection::East, BlockDirection::Up] => RailShape::AscendingEast,
        [BlockDirection::Up, BlockDirection::East] => RailShape::AscendingEast,
        [BlockDirection::West, BlockDirection::Up] => RailShape::AscendingWest,
        [BlockDirection::Up, BlockDirection::West] => RailShape::AscendingWest,
        [BlockDirection::North, BlockDirection::Up] => RailShape::AscendingNorth,
        [BlockDirection::Up, BlockDirection::North] => RailShape::AscendingNorth,
        [BlockDirection::South, BlockDirection::Up] => RailShape::AscendingSouth,
        [BlockDirection::Up, BlockDirection::South] => RailShape::AscendingSouth,
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

async fn can_connect(world: &World, place_pos: &BlockPos, direction: BlockDirection) -> bool {
    let rail_position = place_pos.offset(direction.to_offset());
    let (block, block_state) = world
        .get_block_and_block_state(&rail_position)
        .await
        .unwrap();

    if !block.is_tagged_with("#minecraft:rails").unwrap() {
        // TODO: Handle up and down rails
        return false;
    }

    let rail_props = RailProperties::from_state_id(block_state.id, &block);
    !rail_is_locked(world, &rail_position, &rail_props).await
}

async fn rail_is_locked(
    world: &World,
    rail_position: &BlockPos,
    rail_props: &RailProperties,
) -> bool {
    for direction in get_rail_directions(rail_props.shape) {
        let other_block_position = rail_position.offset(direction.to_offset());
        let (other_block, other_block_state) = world
            .get_block_and_block_state(&other_block_position)
            .await
            .unwrap();

        if !other_block.is_tagged_with("#minecraft:rails").unwrap() {
            // Rails pointing to non-rail blocks are not locked
            return false;
        }

        let other_rail_props = RailProperties::from_state_id(other_block_state.id, &other_block);

        let direction = direction.opposite();
        if !get_rail_directions(other_rail_props.shape)
            .into_iter()
            .any(|d| d == direction)
        {
            // Rails pointing to other rails that are not pointing back are not locked
            return false;
        }
    }

    true
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

        rail_props.shape = if can_connect(world, block_pos, BlockDirection::East).await {
            if can_connect(world, block_pos, BlockDirection::South).await {
                RailShape::SouthEast
            } else if can_connect(world, block_pos, BlockDirection::North).await {
                RailShape::NorthEast
            } else {
                RailShape::EastWest
            }
        } else if can_connect(world, block_pos, BlockDirection::South).await {
            if can_connect(world, block_pos, BlockDirection::West).await {
                RailShape::SouthWest
            } else {
                RailShape::NorthSouth
            }
        } else if can_connect(world, block_pos, BlockDirection::West).await {
            if can_connect(world, block_pos, BlockDirection::North).await {
                RailShape::NorthWest
            } else {
                RailShape::EastWest
            }
        } else if can_connect(world, block_pos, BlockDirection::North).await {
            RailShape::NorthSouth
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
        let rail_props = RailProperties::from_state_id(state_id, block);

        for direction in get_rail_directions(rail_props.shape) {
            let other_block_position = block_pos.offset(direction.to_offset());
            let (other_block, other_block_state) = world
                .get_block_and_block_state(&other_block_position)
                .await
                .unwrap();

            if !other_block.is_tagged_with("#minecraft:rails").unwrap() {
                // Skip non-rail blocks
                continue;
            }

            let mut other_rail_props =
                RailProperties::from_state_id(other_block_state.id, &other_block);

            if get_rail_directions(other_rail_props.shape)
                .into_iter()
                .all(|d| d == direction || d == direction.opposite())
            {
                // Lazy update straight rails pointing to the placed rail
                // TODO handle elevation changes
                continue;
            }

            let mut connections = Vec::with_capacity(2);
            for other_direction in get_rail_directions(other_rail_props.shape) {
                if other_direction == direction.opposite() {
                    // Rails pointing to where the player placed are connected
                    connections.push(other_direction);
                    continue;
                }

                let other_block_position = other_block_position.offset(other_direction.to_offset());
                let (other_block, other_block_state) = world
                    .get_block_and_block_state(&other_block_position)
                    .await
                    .unwrap();

                if !other_block.is_tagged_with("#minecraft:rails").unwrap() {
                    // Rails pointing to non-rail blocks are not connected
                    continue;
                }

                let other_rail_props =
                    RailProperties::from_state_id(other_block_state.id, &other_block);

                if !get_rail_directions(other_rail_props.shape)
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

            other_rail_props.shape = get_rail_shape(&connections);
            world
                .set_block_state(
                    &other_block_position,
                    other_rail_props.to_state_id(&other_block),
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
