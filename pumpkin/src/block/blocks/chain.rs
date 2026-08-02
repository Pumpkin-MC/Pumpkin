use crate::block::BlockFuture;
use crate::block::blocks::copper_weathering;
use crate::block::{BlockBehaviour, OnPlaceArgs, RandomTickArgs};
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block_from_tag;

type ChainLikeProperties = pumpkin_data::block_properties::IronChainLikeProperties;

// Covers the whole `minecraft:chains` tag: iron_chain plus the copper_chain oxidation
// family (unwaxed and waxed). Weathering below only fires for the copper members since
// their oxidation_stages table doesn't include iron_chain.
#[pumpkin_block_from_tag("minecraft:chains")]
pub struct ChainBlock;

impl BlockBehaviour for ChainBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = ChainLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();
            props.r#axis = match args.direction {
                BlockDirection::East | BlockDirection::West => Axis::X,
                BlockDirection::Up | BlockDirection::Down => Axis::Y,
                BlockDirection::North | BlockDirection::South => Axis::Z,
            };

            props.to_state_id(args.block)
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // No tag gate needed: the oxidation_stages table below only contains the
            // copper_chain family, so this is a no-op for iron_chain.
            let current_state_id = args.world.get_block_state_id(args.position);
            let current_props = ChainLikeProperties::from_state_id(current_state_id, args.block);

            let oxidation_stages = [
                &Block::COPPER_CHAIN,
                &Block::EXPOSED_COPPER_CHAIN,
                &Block::WEATHERED_COPPER_CHAIN,
                &Block::OXIDIZED_COPPER_CHAIN,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props = ChainLikeProperties::default(next_block);
                    new_props.r#waterlogged = current_props.r#waterlogged;
                    new_props.r#axis = current_props.r#axis;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}
