use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

type CopperBulbLikeProperties = pumpkin_data::block_properties::CopperBulbLikeProperties;

pub struct CopperBulbBlock;

impl BlockMetadata for CopperBulbBlock {
    fn ids() -> Box<[u16]> {
        [
            Block::COPPER_BULB.id,
            Block::EXPOSED_COPPER_BULB.id,
            Block::WEATHERED_COPPER_BULB.id,
            Block::OXIDIZED_COPPER_BULB.id,
            Block::WAXED_COPPER_BULB.id,
            Block::WAXED_EXPOSED_COPPER_BULB.id,
            Block::WAXED_WEATHERED_COPPER_BULB.id,
            Block::WAXED_OXIDIZED_COPPER_BULB.id,
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
                args.world
                    .play_block_sound(
                        Sound::BlockCopperBulbTurnOn,
                        SoundCategory::Blocks,
                        *args.position,
                    )
                    .await;
                props.powered = true;
            }
            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut props = CopperBulbLikeProperties::from_state_id(state.id, args.block);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;
            if props.powered != is_receiving_power {
                if !props.powered {
                    props.lit = !props.lit;
                    args.world
                        .play_block_sound(
                            if props.lit {
                                Sound::BlockCopperBulbTurnOn
                            } else {
                                Sound::BlockCopperBulbTurnOff
                            },
                            SoundCategory::Blocks,
                            *args.position,
                        )
                        .await;
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
}
