use std::sync::Arc;

use crate::block::pumpkin_block::{
    EmitsRedstonePowerArgs, GetRedstonePowerArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, OnStateReplacedArgs,
};
use async_trait::async_trait;
use pumpkin_data::{
    Block, FacingExt,
    block_properties::{BlockProperties, ObserverLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::{
    block::pumpkin_block::{OnNeighborUpdateArgs, PumpkinBlock},
    world::World,
};

#[pumpkin_block("minecraft:observer")]
pub struct ObserverBlock;

#[async_trait]
impl PumpkinBlock for ObserverBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = ObserverLikeProperties::default(args.block);
        props.facing = args.player.living_entity.entity.get_facing();
        props.to_state_id(args.block)
    }

    async fn on_neighbor_update(&self, _args: OnNeighborUpdateArgs<'_>) {}

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let mut props = ObserverLikeProperties::from_state_id(state.id, args.block);

        if props.powered {
            props.powered = false;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        } else {
            props.powered = true;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            args.world
                .schedule_block_tick(args.block, *args.location, 2, TickPriority::Normal)
                .await;
        }

        Self::update_neighbors(args.world, args.block, args.location, &props).await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = ObserverLikeProperties::from_state_id(args.state_id, args.block);

        if props.facing.to_block_direction() == args.direction && !props.powered {
            Self::schedule_tick(args.world, args.location).await;
        }

        args.state_id
    }

    async fn emits_redstone_power(&self, args: EmitsRedstonePowerArgs<'_>) -> bool {
        let props = ObserverLikeProperties::from_state_id(args.state.id, args.block);
        props.facing.to_block_direction() == args.direction
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = ObserverLikeProperties::from_state_id(args.state.id, args.block);
        if props.facing.to_block_direction() == args.direction && props.powered {
            15
        } else {
            0
        }
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        self.get_weak_redstone_power(args).await
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let props = ObserverLikeProperties::from_state_id(args.old_state_id, args.block);
            if props.powered
                && args
                    .world
                    .is_block_tick_scheduled(args.location, &Block::OBSERVER)
                    .await
            {
                Self::update_neighbors(args.world, args.block, args.location, &props).await;
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
