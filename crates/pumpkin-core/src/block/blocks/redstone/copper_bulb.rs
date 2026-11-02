use std::sync::Arc;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::blocks::weathering_copper::{
    ChangeOverTimeBlock, WeatherState, WeatheringCopper, change_over_time, get_chance_modifier,
    get_first, get_next, get_previous, get_weather_state,
};
use crate::block::{
    BlockBehaviour, BlockMetadata, GetComparatorOutputArgs, OnNeighborUpdateArgs, PlacedArgs,
    RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

type CopperBulbLikeProperties = pumpkin_data::block_properties::CopperBulbLikeProperties;

pub struct CopperBulbBlock;

impl ChangeOverTimeBlock<WeatherState> for CopperBulbBlock {
    fn get_age(&self, block: &pumpkin_data::Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &pumpkin_data::Block) -> Option<&'static pumpkin_data::Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &pumpkin_data::Block) -> Option<&'static pumpkin_data::Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &pumpkin_data::Block) -> Option<&'static pumpkin_data::Block> {
        get_first(block)
    }
}

impl WeatheringCopper for CopperBulbBlock {}

impl BlockMetadata for CopperBulbBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_BULB,
            BlockId::EXPOSED_COPPER_BULB,
            BlockId::WEATHERED_COPPER_BULB,
            BlockId::OXIDIZED_COPPER_BULB,
            BlockId::WAXED_COPPER_BULB,
            BlockId::WAXED_EXPOSED_COPPER_BULB,
            BlockId::WAXED_WEATHERED_COPPER_BULB,
            BlockId::WAXED_OXIDIZED_COPPER_BULB,
        ]
        .into()
    }
}

impl CopperBulbBlock {
    pub fn check_and_flip(
        &self,
        state_id: BlockStateId,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
    ) {
        let mut props = CopperBulbLikeProperties::from_state_id(state_id);
        let is_receiving_power = block_receives_redstone_power(world, position);
        if props.powered != is_receiving_power {
            if !props.powered {
                props.lit = !props.lit;
                world.play_block_sound(
                    if props.lit {
                        Sound::BlockCopperBulbTurnOn
                    } else {
                        Sound::BlockCopperBulbTurnOff
                    },
                    SoundCategory::Blocks,
                    *position,
                );
            }
            props.powered = is_receiving_power;
            world.set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        }
    }
}

impl BlockBehaviour for CopperBulbBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        if Block::from_state_id(args.old_state_id) != args.block {
            self.check_and_flip(args.state_id, args.world, args.position, args.block);
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        self.check_and_flip(state.id, args.world, args.position, args.block);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = CopperBulbLikeProperties::from_state_id(args.state.id);
        if props.lit { Some(15) } else { Some(0) }
    }
}
