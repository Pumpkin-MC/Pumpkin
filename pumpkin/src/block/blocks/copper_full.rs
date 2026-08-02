use crate::block::blocks::copper_weathering;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, RandomTickArgs};
use pumpkin_data::Block;
use pumpkin_data::BlockId;

/// Plain full-cube copper blocks: `copper_block`, `cut_copper` and `chiseled_copper`.
///
/// None of these have any block state properties (unlike bars/chains/grate), so
/// `on_place`/`get_state_for_neighbor_update` don't need overrides - only weathering.
pub struct CopperFullBlock;

impl BlockMetadata for CopperFullBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_BLOCK,
            BlockId::EXPOSED_COPPER,
            BlockId::WEATHERED_COPPER,
            BlockId::OXIDIZED_COPPER,
            BlockId::WAXED_COPPER_BLOCK,
            BlockId::WAXED_EXPOSED_COPPER,
            BlockId::WAXED_WEATHERED_COPPER,
            BlockId::WAXED_OXIDIZED_COPPER,
            BlockId::CUT_COPPER,
            BlockId::EXPOSED_CUT_COPPER,
            BlockId::WEATHERED_CUT_COPPER,
            BlockId::OXIDIZED_CUT_COPPER,
            BlockId::WAXED_CUT_COPPER,
            BlockId::WAXED_EXPOSED_CUT_COPPER,
            BlockId::WAXED_WEATHERED_CUT_COPPER,
            BlockId::WAXED_OXIDIZED_CUT_COPPER,
            BlockId::CHISELED_COPPER,
            BlockId::EXPOSED_CHISELED_COPPER,
            BlockId::WEATHERED_CHISELED_COPPER,
            BlockId::OXIDIZED_CHISELED_COPPER,
            BlockId::WAXED_CHISELED_COPPER,
            BlockId::WAXED_EXPOSED_CHISELED_COPPER,
            BlockId::WAXED_WEATHERED_CHISELED_COPPER,
            BlockId::WAXED_OXIDIZED_CHISELED_COPPER,
        ]
        .into()
    }
}

impl BlockBehaviour for CopperFullBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // This struct covers three independent oxidation families sharing no
            // properties, so try each in turn; try_oxidize_copper is a no-op for a
            // family the current block doesn't belong to.
            const FAMILIES: [[&Block; 4]; 3] = [
                [
                    &Block::COPPER_BLOCK,
                    &Block::EXPOSED_COPPER,
                    &Block::WEATHERED_COPPER,
                    &Block::OXIDIZED_COPPER,
                ],
                [
                    &Block::CUT_COPPER,
                    &Block::EXPOSED_CUT_COPPER,
                    &Block::WEATHERED_CUT_COPPER,
                    &Block::OXIDIZED_CUT_COPPER,
                ],
                [
                    &Block::CHISELED_COPPER,
                    &Block::EXPOSED_CHISELED_COPPER,
                    &Block::WEATHERED_CHISELED_COPPER,
                    &Block::OXIDIZED_CHISELED_COPPER,
                ],
            ];

            for oxidation_stages in &FAMILIES {
                copper_weathering::try_oxidize_copper(
                    args.world,
                    args.position,
                    args.block,
                    oxidation_stages,
                    |next_block| next_block.default_state.id,
                )
                .await;
            }
        })
    }
}
