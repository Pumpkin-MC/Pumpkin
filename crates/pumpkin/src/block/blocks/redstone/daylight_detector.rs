use std::sync::Arc;

use crate::block::entities::daylight_detector::DaylightDetectorBlockEntity;
use pumpkin_data::{Block, block_properties::BlockProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockActionResult, BlockBehaviour, BrokenArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    NormalUseArgs, PlacedArgs,
};
use crate::world::World;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

#[pumpkin_block("minecraft:daylight_detector")]
pub struct DaylightDetectorBlock;

impl BlockBehaviour for DaylightDetectorBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(DaylightDetectorBlockEntity::new(*args.position)));
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.remove_block_entity(args.position);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let player_abilities = args
            .player
            .abilities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !player_abilities.allow_modify_world {
            return BlockActionResult::Pass;
        }

        let state = args.world.get_block_state(args.position);
        let props = DaylightDetectorProperties::from_state_id(state.id, args.block);

        Self::update_inverted(props, args.world, args.position, args.block);

        DaylightDetectorBlockEntity::update_power(args.world, args.position);

        BlockActionResult::Success
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = DaylightDetectorProperties::from_state_id(args.state.id, args.block);
        props.power
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}

impl DaylightDetectorBlock {
    fn update_inverted(
        mut props: DaylightDetectorProperties,
        world: &Arc<World>,
        block_pos: &BlockPos,
        block: &Block,
    ) {
        props.inverted = !props.inverted;

        let state = props.to_state_id(block);

        world.set_block_state(block_pos, state, BlockFlags::NOTIFY_LISTENERS);
    }
}
