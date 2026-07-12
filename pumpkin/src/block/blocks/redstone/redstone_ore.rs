use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, NormalUseArgs, OnEntityCollisionArgs,
    OnEntityStepArgs, OnScheduledTickArgs, registry::BlockActionResult,
};
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, RedstoneOreLikeProperties};
use pumpkin_data::{Block, BlockId, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

pub struct RedstoneOreBlock;

impl BlockMetadata for RedstoneOreBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::REDSTONE_ORE, BlockId::DEEPSLATE_REDSTONE_ORE].into()
    }
}

impl RedstoneOreBlock {
    async fn light_up(world: &Arc<World>, pos: &BlockPos, block: &Block, state: &BlockState) {
        let mut props = RedstoneOreLikeProperties::from_state_id(state.id, block);
        if !props.lit {
            props.lit = true;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        }
        // In vanilla Minecraft, it stays lit for 30 seconds (600 ticks) before scheduled tick turns it off.
        // We use 30 ticks to fit in the u8 parameter range.
        world.schedule_block_tick(block, *pos, 30, TickPriority::Normal);
    }
}

impl BlockBehaviour for RedstoneOreBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            Self::light_up(args.world, args.position, args.block, state).await;
            BlockActionResult::Success
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            Self::light_up(args.world, args.position, args.block, state).await;
        })
    }

    fn on_entity_step<'a>(&'a self, args: OnEntityStepArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            Self::light_up(args.world, args.position, args.block, state).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = RedstoneOreLikeProperties::from_state_id(state.id, args.block);
            if props.lit {
                props.lit = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}
