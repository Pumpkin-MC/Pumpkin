use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::BlockState;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::DoorHinge;
use pumpkin_data::block::OakDoorProps;
use pumpkin_data::block::VerticalHalf;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

async fn toggle_door(world: &World, block_pos: &BlockPos) {
    let state_id = world.get_block_state_id(block_pos).await;
    if let Ok(state_id) = state_id {
        let mut door_props = OakDoorProps::from_state_id(state_id).unwrap();
        door_props.open = door_props.open.flip();

        let other_half = match door_props.half {
            VerticalHalf::Upper => BlockDirection::Down,
            VerticalHalf::Lower => BlockDirection::Up,
        };
        let other_pos = block_pos.offset(other_half.to_offset());

        let other_state_id = world.get_block_state_id(&other_pos).await.unwrap();
        let mut other_door_props = OakDoorProps::from_state_id(other_state_id).unwrap();
        other_door_props.open = door_props.open;

        world
            .set_block_state(block_pos, door_props.to_state_id())
            .await;
        world
            .set_block_state(&other_pos, other_door_props.to_state_id())
            .await;
    }
}

#[pumpkin_block("minecraft:oak_door")]
pub struct OakDoorBlock;

#[async_trait]
impl PumpkinBlock for OakDoorBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &CardinalDirection,
        _other: bool,
    ) -> u16 {
        // TODO: Door hinge
        let mut door_props = OakDoorProps::from_state_id(block.default_state_id).unwrap();
        door_props.half = VerticalHalf::Lower;
        door_props.facing = *player_direction;
        door_props.hinge = DoorHinge::Left;

        door_props.to_state_id()
    }

    async fn can_place(
        &self,
        _server: &Server,
        world: &World,
        _block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _player_direction: &CardinalDirection,
    ) -> bool {
        if world
            .get_block_state(&block_pos.offset(BlockDirection::Up.to_offset()))
            .await
            .is_ok_and(|state| state.replaceable)
        {
            return true;
        }
        false
    }

    async fn placed(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &World,
    ) {
        let mut upper_door_props =
            OakDoorProps::from_state_id(world.get_block_state_id(&location).await.unwrap())
                .unwrap();

        upper_door_props.half = VerticalHalf::Upper;

        world
            .set_block_state(
                &location.offset(BlockDirection::Up.to_offset()),
                upper_door_props.to_state_id(),
            )
            .await;
    }

    async fn broken(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        server: &Server,
        world: Arc<World>,
        state: BlockState,
    ) {
        let door_props = OakDoorProps::from_state_id(state.id).unwrap();

        let other_half = match door_props.half {
            VerticalHalf::Upper => BlockDirection::Down,
            VerticalHalf::Lower => BlockDirection::Up,
        };

        let other_pos = location.offset(other_half.to_offset());

        if let Ok(other_block) = world.get_block(&other_pos).await {
            if other_block.id == block.id {
                world
                    .break_block(&other_pos, None, true, Some(server))
                    .await;
            }
        }
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &World,
    ) -> BlockActionResult {
        toggle_door(world, &location).await;
        BlockActionResult::Consume
    }

    async fn normal_use(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &World,
    ) {
        toggle_door(world, &location).await;
    }
}
