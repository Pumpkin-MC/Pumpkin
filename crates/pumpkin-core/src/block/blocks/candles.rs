use pumpkin_data::{
    BlockDirection, BlockStateId,
    block_properties::CandleLikeProperties,
    entity::EntityPose,
    fluid::Fluid,
    game_event::GameEvent,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

use crate::block::{GetStateForNeighborUpdateArgs, can_support_center};
use crate::{
    block::{
        BlockIsReplacing,
        registry::BlockActionResult,
        {BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, NormalUseArgs, OnPlaceArgs},
    },
    entity::EntityBase,
};

#[pumpkin_block_from_tag("minecraft:candles")]
pub struct CandleBlock;

impl BlockBehaviour for CandleBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if let BlockIsReplacing::Itself(state_id) = args.replacing {
            let mut properties = CandleLikeProperties::from_state_id(state_id);
            if properties.candles < 4 {
                properties.candles += 1;
            }
            return properties.to_state_id(args.block);
        }

        let mut properties = CandleLikeProperties::default(args.block);
        properties.waterlogged = args.replacing.water_source();
        properties.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let mut properties = CandleLikeProperties::from_state_id(state_id);

        if args.player.may_build() && properties.lit {
            properties.lit = false;
            args.world.set_block_state(
                args.position,
                properties.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            args.world.play_sound(
                Sound::BlockCandleExtinguish,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
            );
            args.world.emit_game_event(
                GameEvent::BlockChange.name(),
                args.position.to_centered_f64(),
            );
            return BlockActionResult::Success;
        }

        BlockActionResult::Pass
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let b = BlockAccessor::get_block(args.world, args.position);
        args.player.get_entity().pose.load() != EntityPose::Crouching
            && CandleLikeProperties::from_state_id(args.state_id).candles < 4
            && args.block.id == b.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let properties = CandleLikeProperties::from_state_id(args.state_id);
        if properties.waterlogged {
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

fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    can_support_center(block_accessor, &position.down(), BlockDirection::Up)
}
