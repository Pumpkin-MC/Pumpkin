use crate::block::blocks::copper_weathering;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs, RandomTickArgs,
};
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_world::world::BlockFlags;

type CopperBulbLikeProperties = pumpkin_data::block_properties::CopperBulbLikeProperties;

pub struct CopperBulbBlock;

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
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CopperBulbLikeProperties::default(args.block);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;
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
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = CopperBulbLikeProperties::from_state_id(state.id, args.block);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;
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
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let oxidation_stages = [
                &pumpkin_data::Block::COPPER_BULB,
                &pumpkin_data::Block::EXPOSED_COPPER_BULB,
                &pumpkin_data::Block::WEATHERED_COPPER_BULB,
                &pumpkin_data::Block::OXIDIZED_COPPER_BULB,
            ];

            let current_state_id = args.world.get_block_state_id(args.position);
            let current_props =
                CopperBulbLikeProperties::from_state_id(current_state_id, args.block);

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props = CopperBulbLikeProperties::default(next_block);
                    new_props.lit = current_props.lit;
                    new_props.powered = current_props.powered;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}
