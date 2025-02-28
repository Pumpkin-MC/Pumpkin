use crate::{
    block::properties::{BlockPropertyMetadata, facing::Facing, half::evaluate_half},
    world::World,
};
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, Direction};

#[block_property("shape")]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

#[async_trait]
impl BlockProperty for StairShape {
    async fn on_place(
        &self,
        world: &World,
        _block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        properties: &BlockProperties,
        _other: bool,
    ) -> String {
        let block_half = evaluate_half(*face, use_item_on);
        let (front_block_pos, back_block_pos) = calculate_positions(player_direction, block_pos);

        let front_block_and_state = world.get_block_and_block_state(&front_block_pos).await;
        let back_block_and_state = world.get_block_and_block_state(&back_block_pos).await;

        match front_block_and_state {
            Ok((block, state)) => {
                if block.name.ends_with("stairs") {
                    log::debug!("Block in front is a stair block");

                    let key = state.id - block.states[0].id;
                    if let Some(properties) = properties.property_mappings.get(&key) {
                        if properties.contains(&Self::Straight.value())
                            && properties.contains(&block_half)
                        {
                            let is_facing_north = properties.contains(&Facing::North.value());
                            let is_facing_west = properties.contains(&Facing::West.value());
                            let is_facing_south = properties.contains(&Facing::South.value());
                            let is_facing_east = properties.contains(&Facing::East.value());

                            if (is_facing_north && *player_direction == Direction::West)
                                || (is_facing_west && *player_direction == Direction::South)
                                || (is_facing_south && *player_direction == Direction::East)
                                || (is_facing_east && *player_direction == Direction::North)
                            {
                                return Self::OuterRight.value();
                            }

                            if (is_facing_north && *player_direction == Direction::East)
                                || (is_facing_west && *player_direction == Direction::North)
                                || (is_facing_south && *player_direction == Direction::West)
                                || (is_facing_east && *player_direction == Direction::South)
                            {
                                return Self::OuterLeft.value();
                            }
                        }
                    }
                } else {
                    log::debug!("Block to the left is not a stair block");
                }
            }
            Err(_) => {
                log::debug!("There is no block to the left");
            }
        }

        match back_block_and_state {
            Ok((block, state)) => {
                if block.name.ends_with("stairs") {
                    log::debug!("Block in back is a stair block");

                    let key = state.id - block.states[0].id;
                    if let Some(properties) = properties.property_mappings.get(&key) {
                        if properties.contains(&Self::Straight.value())
                            && properties.contains(&block_half)
                        {
                            let is_facing_north = properties.contains(&Facing::North.value());
                            let is_facing_west = properties.contains(&Facing::West.value());
                            let is_facing_south = properties.contains(&Facing::South.value());
                            let is_facing_east = properties.contains(&Facing::East.value());

                            if (is_facing_north && *player_direction == Direction::West)
                                || (is_facing_west && *player_direction == Direction::South)
                                || (is_facing_south && *player_direction == Direction::East)
                                || (is_facing_east && *player_direction == Direction::North)
                            {
                                return Self::InnerRight.value();
                            }

                            if (is_facing_north && *player_direction == Direction::East)
                                || (is_facing_west && *player_direction == Direction::North)
                                || (is_facing_south && *player_direction == Direction::West)
                                || (is_facing_east && *player_direction == Direction::South)
                            {
                                return Self::InnerLeft.value();
                            }
                        }
                    }
                } else {
                    log::debug!("Block to the right is not a stair block");
                }
            }
            Err(_) => {
                log::debug!("There is no block to the right");
            }
        }

        // TODO: We currently don't notify adjacent stair blocks to update their shape after placement.
        //       We should implement a block update mechanism (e.g., tracking state changes and triggering
        //       a server-wide or chunk-level update) so that neighbors properly recalculate their shape.

        Self::Straight.value()
    }
}

fn calculate_positions(player_direction: &Direction, block_pos: &BlockPos) -> (BlockPos, BlockPos) {
    match player_direction {
        Direction::North => (
            BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y,
                block_pos.0.z - 1,
            )),
            BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y,
                block_pos.0.z + 1,
            )),
        ),
        Direction::South => (
            BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y,
                block_pos.0.z + 1,
            )),
            BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y,
                block_pos.0.z - 1,
            )),
        ),
        Direction::East => (
            BlockPos(Vector3::new(
                block_pos.0.x + 1,
                block_pos.0.y,
                block_pos.0.z,
            )),
            BlockPos(Vector3::new(
                block_pos.0.x - 1,
                block_pos.0.y,
                block_pos.0.z,
            )),
        ),
        Direction::West => (
            BlockPos(Vector3::new(
                block_pos.0.x - 1,
                block_pos.0.y,
                block_pos.0.z,
            )),
            BlockPos(Vector3::new(
                block_pos.0.x + 1,
                block_pos.0.y,
                block_pos.0.z,
            )),
        ),
    }
}
