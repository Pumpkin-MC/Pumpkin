use crate::block::blocks::copper_weathering;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs, RandomTickArgs};
use pumpkin_data::Block;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;

// Copper grate has no vanilla analogue outside the copper family (there is no plain
// "grate" block), so unlike bars/chains this can't ride an existing tag shared with a
// non-copper block. Its state layout is a single `waterlogged` bool, matching
// `MangroveRootsLikeProperties` (also used by conduit.rs for the same shape).
type CopperGrateProperties = pumpkin_data::block_properties::MangroveRootsLikeProperties;

pub struct CopperGrateBlock;

impl BlockMetadata for CopperGrateBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_GRATE,
            BlockId::EXPOSED_COPPER_GRATE,
            BlockId::WEATHERED_COPPER_GRATE,
            BlockId::OXIDIZED_COPPER_GRATE,
            BlockId::WAXED_COPPER_GRATE,
            BlockId::WAXED_EXPOSED_COPPER_GRATE,
            BlockId::WAXED_WEATHERED_COPPER_GRATE,
            BlockId::WAXED_OXIDIZED_COPPER_GRATE,
        ]
        .into()
    }
}

impl BlockBehaviour for CopperGrateBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CopperGrateProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();

            props.to_state_id(args.block)
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let current_state_id = args.world.get_block_state_id(args.position);
            let current_props = CopperGrateProperties::from_state_id(current_state_id, args.block);

            let oxidation_stages = [
                &Block::COPPER_GRATE,
                &Block::EXPOSED_COPPER_GRATE,
                &Block::WEATHERED_COPPER_GRATE,
                &Block::OXIDIZED_COPPER_GRATE,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props = CopperGrateProperties::default(next_block);
                    new_props.r#waterlogged = current_props.r#waterlogged;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}
