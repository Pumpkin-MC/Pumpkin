use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::LightLikeProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{GameMode, PermissionLvl};
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, OnPlaceArgs};

#[pumpkin_block("minecraft:light")]
pub struct LightBlock;

impl BlockBehaviour for LightBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LightLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.gamemode.load() == GameMode::Creative
            && args.player.permission_lvl.load() >= PermissionLvl::Two
        {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = LightLikeProperties::from_state_id(state_id);
            props.level = (props.level + 1) % 16;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            BlockActionResult::SuccessServer
        } else {
            BlockActionResult::Consume
        }
    }
}
