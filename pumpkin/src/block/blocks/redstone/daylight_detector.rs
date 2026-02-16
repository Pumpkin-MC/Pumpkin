use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockActionResult, BlockBehaviour, BlockFuture, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    Integer0To15, NormalUseArgs, OnScheduledTickArgs,
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

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async {
            let state = args.world.get_block_state(args.position).await;
            let props = DaylightDetectorProperties::from_state_id(state.id, args.block);

            self.update_state(props, args.world, args.position, args.block)
                .await;
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
    async fn update_state(
        &self,
        props: DaylightDetectorProperties,
        world: &Arc<World>,
        block_pos: &BlockPos,
        block: &Block,
    ) {
        // TODO: finish power calculation
        // for this we need to get the ambient darkness which is not implemented yet in the light engine
        // and the sun angle attribute
        let mut props = props;
        let sky_light_level = world
            .level
            .light_engine
            .get_sky_light_level(&world.level, block_pos)
            .await
            .unwrap();
        let ambient_darkness = 0; // TODO
        let effective_sky_light = sky_light_level - ambient_darkness;
        // let sun_angle;
        let inverted = props.inverted;

        let mut power = 0;
        if inverted {
            power = 15 - effective_sky_light;
        } else if effective_sky_light > 0 {
            // TODO:
            // some math
            // see source code: net.minecraft.block.DaylightDetectorBlock.java
        }

        let power = Integer0To15::from_index(power.clamp(0, 15).into());
        if power != props.power {
            props.power = power;
            let state = props.to_state_id(block);
            world
                .set_block_state(block_pos, state, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }

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
