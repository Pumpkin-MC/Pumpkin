use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, EmitsRedstonePowerArgs, GetRedstonePowerArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, Facing, LightningRodLikeProperties};
use pumpkin_data::{BlockDirection, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:lightning_rod")]
pub struct LightningRodBlock;

impl LightningRodBlock {
    pub async fn trigger(world: &Arc<World>, pos: &BlockPos) {
        let (block, state_id) = world.get_block_and_state_id(pos);
        let mut props = LightningRodLikeProperties::from_state_id(state_id, block);
        if !props.powered {
            props.powered = true;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;

            Self::update_neighbors(world, pos, &props).await;

            // In vanilla, it stays powered for 8 ticks (4 redstone ticks) before scheduled tick turns it off.
            world.schedule_block_tick(block, *pos, 8, TickPriority::Normal);
        }
    }

    async fn update_neighbors(
        world: &Arc<World>,
        pos: &BlockPos,
        props: &LightningRodLikeProperties,
    ) {
        world.update_neighbors(pos, None).await;
        // The block it is attached to is in the opposite of the facing direction
        let attached_pos = pos.offset(props.facing.opposite().to_block_direction().to_offset());
        world.update_neighbors(&attached_pos, None).await;
    }
}

impl BlockBehaviour for LightningRodBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let facing = match args.direction {
                BlockDirection::North => Facing::North,
                BlockDirection::South => Facing::South,
                BlockDirection::East => Facing::East,
                BlockDirection::West => Facing::West,
                BlockDirection::Up => Facing::Up,
                BlockDirection::Down => Facing::Down,
            };
            let mut props = LightningRodLikeProperties::default(args.block);
            props.facing = facing;
            props.to_state_id(args.block)
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = LightningRodLikeProperties::from_state_id(args.state.id, args.block);
            if props.powered { 15 } else { 0 }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = LightningRodLikeProperties::from_state_id(args.state.id, args.block);
            // It emits strong power only in its facing direction (the direction pointing outward)
            if props.powered && props.facing.to_block_direction() == args.direction {
                15
            } else {
                0
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = LightningRodLikeProperties::from_state_id(state.id, args.block);
            if props.powered {
                props.powered = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                Self::update_neighbors(args.world, args.position, &props).await;
            }
        })
    }
}
