use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::entities::daylight_detector::DaylightDetectorBlockEntity;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockActionResult, BlockBehaviour, BlockFuture, BrokenArgs, EmitsRedstonePowerArgs,
    GetRedstonePowerArgs, NormalUseArgs, PlacedArgs,
};
use crate::world::World;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

#[pumpkin_block("minecraft:daylight_detector")]
pub struct DaylightDetectorBlock;

impl BlockBehaviour for DaylightDetectorBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .add_block_entity(Arc::new(DaylightDetectorBlockEntity::new(*args.position)))
                .await;
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.remove_block_entity(args.position).await;
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async {
            let state = args.world.get_block_state(args.position).await;
            let props = DaylightDetectorProperties::from_state_id(state.id, args.block);

            self.update_inverted(props, args.world, args.position, args.block)
                .await;

            DaylightDetectorBlockEntity::update_power(args.world, args.position).await;

            BlockActionResult::Success
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let props = DaylightDetectorProperties::from_state_id(state.id, args.block);

            props.power.to_index() as u8
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
