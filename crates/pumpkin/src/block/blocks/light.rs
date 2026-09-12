use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::LightLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockIsReplacing, CanUpdateAtArgs, GetStateForNeighborUpdateArgs,
    NormalUseArgs, OnPlaceArgs,
};

#[pumpkin_block("minecraft:light")]
pub struct LightBlock;

impl BlockBehaviour for LightBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = if let BlockIsReplacing::Itself(state_id) = args.replacing {
            let mut p = LightLikeProperties::from_state_id(state_id);
            p.level = (p.level + 1) % 16;
            p
        } else {
            LightLikeProperties::default(args.block)
        };
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        args.player.gamemode.load() == pumpkin_util::GameMode::Creative
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.gamemode.load() == pumpkin_util::GameMode::Creative {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = LightLikeProperties::from_state_id(state_id);
            props.level = (props.level + 1) % 16;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            BlockActionResult::SuccessServer
        } else {
            BlockActionResult::Consume
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = LightLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }
}
