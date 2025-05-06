use std::sync::Arc;

use crate::{block::BlockIsReplacing, entity::player::Player};
use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, ObserverLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    block::{BlockDirection, FacingExt},
    chunk::TickPriority,
};

use crate::{
    block::pumpkin_block::PumpkinBlock,
    server::Server,
    world::{BlockFlags, World},
};

#[pumpkin_block("minecraft:observer")]
pub struct ObserverBlock;

#[async_trait]
impl PumpkinBlock for ObserverBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = ObserverLikeProperties::default(block);
        props.facing = player.living_entity.entity.get_facing();
        props.to_state_id(block)
    }

    async fn on_neighbor_update(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
        let state = world.get_block_state(block_pos).await.unwrap();
        let mut props = ObserverLikeProperties::from_state_id(state.id, block);

        if props.powered {
            props.powered = false;
            world
                .set_block_state(
                    block_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        } else {
            props.powered = true;
            world
                .set_block_state(
                    block_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            world
                .schedule_block_tick(block, *block_pos, 2, TickPriority::Normal)
                .await;
        }

        Self::update_neighbors(world, block, block_pos, &props).await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        block_pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let props = ObserverLikeProperties::from_state_id(state, block);

        if props.facing.to_block_direction() == direction && !props.powered {
            Self::schedule_tick(world, block_pos).await;
        }

        state
    }

    async fn emits_redstone_power(
        &self,
        block: &Block,
        state: &BlockState,
        direction: BlockDirection,
    ) -> bool {
        let props = ObserverLikeProperties::from_state_id(state.id, block);
        props.facing.to_block_direction() == direction
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let props = ObserverLikeProperties::from_state_id(state.id, block);
        if props.facing.to_block_direction() == direction && props.powered {
            15
        } else {
            0
        }
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        self.get_weak_redstone_power(block, world, block_pos, state, direction)
            .await
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        location: BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        if !moved {
            let props = ObserverLikeProperties::from_state_id(old_state_id, block);
            if props.powered
                && world
                    .is_block_tick_scheduled(&location, &Block::OBSERVER)
                    .await
            {
                Self::update_neighbors(world, block, &location, &props).await;
            }
        }
    }
}

impl ObserverBlock {
    async fn update_neighbors(
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        props: &ObserverLikeProperties,
    ) {
        let facing = props.facing;
        let opposite_facing_pos =
            block_pos.offset(facing.to_block_direction().opposite().to_offset());
        world.update_neighbor(&opposite_facing_pos, block).await;
        world
            .update_neighbors(&opposite_facing_pos, Some(facing.to_block_direction()))
            .await;
    }

    async fn schedule_tick(world: &World, block_pos: &BlockPos) {
        if world
            .is_block_tick_scheduled(block_pos, &Block::OBSERVER)
            .await
        {
            return;
        }
        world
            .schedule_block_tick(&Block::OBSERVER, *block_pos, 2, TickPriority::Normal)
            .await;
    }
}
