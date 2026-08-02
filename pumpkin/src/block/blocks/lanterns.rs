use crate::block::blocks::copper_weathering;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_data::{BlockDirection, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:lanterns")]
pub struct LanternBlock;

impl BlockBehaviour for LanternBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                pumpkin_data::block_properties::LanternLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();

            let block_up_state = args.world.get_block_state(&args.position.up());
            if block_up_state.is_center_solid(BlockDirection::Down) {
                props.r#hanging = true;
            }

            props.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| can_place_at(world, args.position))
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // No tag gate needed: the oxidation_stages table below only contains the
            // copper lantern family, so this is a no-op for every other lantern type.

            let current_state_id = args.world.get_block_state_id(args.position);
            let lantern_props =
                pumpkin_data::block_properties::LanternLikeProperties::from_state_id(
                    current_state_id,
                    args.block,
                );

            let oxidation_stages = [
                &Block::COPPER_LANTERN,
                &Block::EXPOSED_COPPER_LANTERN,
                &Block::WEATHERED_COPPER_LANTERN,
                &Block::OXIDIZED_COPPER_LANTERN,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props =
                        pumpkin_data::block_properties::LanternLikeProperties::default(next_block);
                    new_props.r#hanging = lantern_props.r#hanging;
                    new_props.r#waterlogged = lantern_props.r#waterlogged;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}

fn can_place_at(world: &World, position: &BlockPos) -> bool {
    //idk why this don't update with .is_center_solid so this is a 'temporary patch'
    if world
        .get_block(&position.down())
        .has_tag(&tag::Block::C_FENCE_GATES)
    {
        let fence_gate_props =
            pumpkin_data::block_properties::OakFenceGateLikeProperties::from_state_id(
                world.get_block_state_id(&position.down()),
                world.get_block(&position.down()),
            );

        if fence_gate_props.open {
            return false;
        }
    }
    let (block_down, block_down_state) = world.get_block_and_state(&position.down());
    let block_up_state = world.get_block_state(&position.up());
    block_down_state.is_center_solid(BlockDirection::Up)
        || block_up_state.is_center_solid(BlockDirection::Down)
        || block_down.has_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER)
}
