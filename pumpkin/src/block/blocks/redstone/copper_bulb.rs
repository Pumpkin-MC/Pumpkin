use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

type CopperBulbLikeProperties = pumpkin_data::block_properties::CopperBulbLikeProperties;

pub struct CopperBulbBlock;

impl BlockMetadata for CopperBulbBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            "copper_bulb",
            "exposed_copper_bulb",
            "weathered_copper_bulb",
            "oxidized_copper_bulb",
            "waxed_copper_bulb",
            "waxed_exposed_copper_bulb",
            "waxed_weathered_copper_bulb",
            "waxed_oxidized_copper_bulb",
        ]
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
