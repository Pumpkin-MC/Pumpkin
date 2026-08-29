use crate::block::{
    BlockBehaviour, BlockMetadata, NormalUseArgs, OnEntityCollisionArgs, OnEntityStepArgs,
    RandomTickArgs, registry::BlockActionResult,
};
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, RedstoneOreLikeProperties};
use pumpkin_data::{Block, BlockId, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

pub struct RedstoneOreBlock;

impl BlockMetadata for RedstoneOreBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::REDSTONE_ORE, BlockId::DEEPSLATE_REDSTONE_ORE].into()
    }
}

impl RedstoneOreBlock {
    fn light_up(world: &Arc<World>, pos: &BlockPos, block: &Block, state: &BlockState) {
        let mut props = RedstoneOreLikeProperties::from_state_id(state.id, block);
        if !props.lit {
            props.lit = true;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        }
    }
}

impl BlockBehaviour for RedstoneOreBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        Self::light_up(args.world, args.position, args.block, state);
        BlockActionResult::Success
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        Self::light_up(args.world, args.position, args.block, state);
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        Self::light_up(args.world, args.position, args.block, state);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let mut props = RedstoneOreLikeProperties::from_state_id(state.id, args.block);

        if props.lit {
            props.lit = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}
