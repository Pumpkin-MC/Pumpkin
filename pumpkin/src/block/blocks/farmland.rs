use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::RandomTickArgs;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::EnumVariants;
use pumpkin_data::block_properties::FarmlandLikeProperties;
use pumpkin_data::block_properties::Integer0To7;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type FarmlandProperties = FarmlandLikeProperties;

#[pumpkin_block("minecraft:farmland")]
pub struct FarmlandBlock;

#[async_trait]
impl BlockBehaviour for FarmlandBlock {
    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        // TODO: push up entities
        args.world
            .set_block_state(
                args.position,
                Block::DIRT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.position).await {
            return Block::DIRT.default_state.id;
        }
        args.block.default_state.id
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Up && !can_place_at(args.world, args.position).await {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position).await
    }

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        // TODO: add rain check. Remember to check which one is most optimized.
        if is_water_nearby(args.world, args.position).await {
            let mut props = FarmlandProperties::default(args.block);
            props.moisture = Integer0To7::L7;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
        } else {
            let state_id = args.world.get_block_state_id(args.position).await;
            let mut props = FarmlandProperties::from_state_id(state_id, args.block);
            if props.moisture == Integer0To7::L0 {
                if !args
                    .world
                    .get_block(&args.position.up())
                    .await
                    .is_tagged_with_by_tag(&tag::Block::MINECRAFT_MAINTAINS_FARMLAND)
                {
                    //TODO push entities up
                    args.world
                        .set_block_state(
                            args.position,
                            Block::DIRT.default_state.id,
                            BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                }
            } else {
                props.moisture = Integer0To7::from_index(props.moisture.to_index() - 1);
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        }
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let state = world.get_block_state(&block_pos.up()).await;
    !state.is_solid() // TODO: add fence gate block
}

async fn is_water_nearby(world: &Arc<World>, block_pos: &BlockPos) -> bool {
    for dx in -4..=4 {
        for dy in 0..=1 {
            for dz in -4..=4 {
                let check_pos = block_pos.offset(Vector3 {
                    x: dx,
                    y: dy,
                    z: dz,
                });
                //TODO this should use tag water. It does not seem to work rn.
                if world.get_block(&check_pos).await == &Block::WATER {
                    return true;
                }
            }
        }
    }
    false
}
