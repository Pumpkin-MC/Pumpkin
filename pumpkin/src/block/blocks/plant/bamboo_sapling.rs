use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BambooLeaves, BambooLikeProperties, BlockProperties},
    item::Item,
    tag::Taggable,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnNeighborUpdateArgs, UseWithItemArgs, blocks::plant::PlantBlockBase,
    registry::BlockActionResult,
};

#[pumpkin_block("minecraft:bamboo_sapling")]
pub struct BambooSaplingBlock;

impl BlockBehaviour for BambooSaplingBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, &args.position.down())
                .await
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let lock = args.item_stack.lock().await;
            if lock.get_item() == &Item::BONE_MEAL {
                let mut props_new = BambooLikeProperties::from_state_id(
                    Block::BAMBOO.default_state.id,
                    &Block::BAMBOO,
                );
                props_new.leaves = BambooLeaves::Small;
                args.world
                    .set_block_state(
                        &args.position.up(),
                        props_new.to_state_id(&Block::BAMBOO),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                return BlockActionResult::Success;
            }
            BlockActionResult::Pass
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.block == &Block::BAMBOO_SAPLING
                && args.world.get_block(&args.position.up()).await == &Block::BAMBOO
            {
                args.world
                    .set_block_state(
                        args.position,
                        Block::BAMBOO.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position).await {
                return Block::AIR.default_state.id;
            }
            if args.direction == BlockDirection::Up
                && args.world.get_block(args.neighbor_position).await == &Block::BAMBOO
            {
                return Block::BAMBOO.default_state.id;
            }
            args.state_id
        })
    }

    fn random_tick<'a>(&'a self, args: crate::block::RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_above = args.world.get_block_state(&args.position.up()).await;
            if !state_above.is_air() || rand::rng().random_range(0..3) > 0 {
                return;
            }
            let mut props_new =
                BambooLikeProperties::from_state_id(Block::BAMBOO.default_state.id, &Block::BAMBOO);
            props_new.leaves = BambooLeaves::Small;
            args.world
                .set_block_state(
                    &args.position.up(),
                    props_new.to_state_id(&Block::BAMBOO),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        })
    }
}

impl PlantBlockBase for BambooSaplingBlock {
    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }

    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BAMBOO_PLANTABLE_ON)
    }
}
