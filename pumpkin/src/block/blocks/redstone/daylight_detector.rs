use std::sync::Arc;

use pumpkin_data::{Block, block_properties::BlockProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockActionResult, BlockBehaviour, BlockFuture, EmitsRedstonePowerArgs, NormalUseArgs,
};
use crate::world::World;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

#[pumpkin_block("minecraft:daylight_detector")]
pub struct DaylightDetectorBlock;

impl BlockBehaviour for DaylightDetectorBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async {
            let state = args.world.get_block_state(args.position).await;
            let props = DaylightDetectorProperties::from_state_id(state.id, args.block);

            self.update_inverted(props, args.world, args.position, args.block)
                .await;

            BlockActionResult::Success
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }
}

impl DaylightDetectorBlock {
    async fn update_inverted(
        &self,
        props: DaylightDetectorProperties,
        world: &Arc<World>,
        block_pos: &BlockPos,
        block: &Block,
    ) {
        let mut props = props;
        props.inverted = !props.inverted;

        let state = props.to_state_id(block);

        world
            .set_block_state(block_pos, state, BlockFlags::NOTIFY_LISTENERS)
            .await;
    }
}
