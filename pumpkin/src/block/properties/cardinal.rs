use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("down")]
pub enum Down {
    True,
    False,
    Tall,
    Low,
    None,
}
#[block_property("east")]
pub enum East {
    True,
    False,
    Tall,
    Low,
    None,
}
#[block_property("north")]
pub enum North {
    True,
    False,
    Tall,
    Low,
    None,
}
#[block_property("south")]
pub enum South {
    True,
    False,
    Tall,
    Low,
    None,
}
#[block_property("up")]
pub enum Up {
    True,
    False,
    Tall,
    Low,
    None,
}
#[block_property("west")]
pub enum West {
    True,
    False,
    Tall,
    Low,
    None,
}

pub async fn evaluate_fence_direction(
    world: &World,
    placed_block: &Block,
    face: &BlockDirection,
    block_pos: &BlockPos,
) -> String {
    let other_side_block = BlockPos(block_pos.0.add(&face.to_offset()));
    let block = world.get_block(&other_side_block).await.unwrap();
    if placed_block.name.ends_with("_wall") {
        if face == &BlockDirection::Top {
            if block.id != 0 {
                return North::True.value();
            }

            let mut x = 0u8;
            let mut z = 0u8;
            for side in &[
                BlockDirection::North,
                BlockDirection::East,
                BlockDirection::South,
                BlockDirection::West,
            ] {
                let other_side_block = BlockPos(block_pos.0.add(&side.to_offset()));
                let block = world.get_block(&other_side_block).await.unwrap();
                if block.id != 0 {
                    if *side == BlockDirection::North || *side == BlockDirection::South {
                        x += 1;
                    } else {
                        z += 1;
                    }
                }
            }
            if (z == 0 || z == 2) && x == 2 || (x == 0 && z == 2) {
                return North::False.value();
            }

            return North::True.value();
        }
        if block.id != 0 {
            let other_side_block = BlockPos(block_pos.0.add(&Vector3::new(0, 1, 0)));
            let block = world.get_block(&other_side_block).await.unwrap();
            if block.id != 0 {
                return North::Tall.value();
            }
            return North::Low.value();
        }
        return North::None.value();
    }
    if block.id != 0 {
        return North::True.value();
    }
    North::False.value()
}

#[async_trait]
impl BlockProperty for Down {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::Bottom, block_pos).await
    }
}
#[async_trait]
impl BlockProperty for East {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::East, block_pos).await
    }
}
#[async_trait]
impl BlockProperty for North {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::North, block_pos).await
    }
}
#[async_trait]
impl BlockProperty for South {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::South, block_pos).await
    }
}
#[async_trait]
impl BlockProperty for Up {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::Top, block_pos).await
    }
}
#[async_trait]
impl BlockProperty for West {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_fence_direction(world, block, &BlockDirection::West, block_pos).await
    }
}
