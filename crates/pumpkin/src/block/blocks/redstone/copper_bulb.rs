use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::blocks::weathering_copper::{
    ChangeOverTimeBlock, WeatherState, WeatheringCopper, change_over_time, get_chance_modifier,
    get_first, get_next, get_previous, get_weather_state,
};
use crate::block::{
    BlockBehaviour, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs, RandomTickArgs,
};
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
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

impl BlockBehaviour for CopperBulbBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CopperBulbLikeProperties::default(args.block);
        let is_receiving_power = block_receives_redstone_power(args.world, args.position);
        if is_receiving_power {
            props.lit = true;
            args.world.play_block_sound(
                Sound::BlockCopperBulbTurnOn,
                SoundCategory::Blocks,
                *args.position,
            );
            props.powered = true;
        }
        props.to_state_id(args.block)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state = args.world.get_block_state(args.position);
            let mut props = CopperBulbLikeProperties::from_state_id(state.id, args.block);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position);
            if props.powered != is_receiving_power {
                if !props.powered {
                    props.lit = !props.lit;
                    args.world.play_block_sound(
                        if props.lit {
                            Sound::BlockCopperBulbTurnOn
                        } else {
                            Sound::BlockCopperBulbTurnOff
                        },
                        SoundCategory::Blocks,
                        *args.position,
                    );
                }
                props.powered = is_receiving_power;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}
