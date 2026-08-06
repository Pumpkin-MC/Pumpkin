use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SlabType;
use pumpkin_macros::pumpkin_block_from_tag;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::BlockIsReplacing;
use crate::block::CanUpdateAtArgs;
use crate::block::OnPlaceArgs;
use crate::block::RandomTickArgs;
use crate::block::blocks::copper_weathering;

type SlabProperties = pumpkin_data::block_properties::ResinBrickSlabLikeProperties;

#[pumpkin_block_from_tag("minecraft:slabs")]
pub struct SlabBlock;

impl BlockBehaviour for SlabBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if let BlockIsReplacing::Itself(state_id) = args.replacing {
                let mut slab_props = SlabProperties::from_state_id(state_id, args.block);
                slab_props.r#type = SlabType::Double;
                slab_props.waterlogged = false;
                return slab_props.to_state_id(args.block);
            }

            let mut slab_props = SlabProperties::default(args.block);
            slab_props.waterlogged = args.replacing.water_source();
            slab_props.r#type = match args.direction {
                BlockDirection::Up => SlabType::Top,
                BlockDirection::Down => SlabType::Bottom,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0..0.5 => SlabType::Bottom,
                    _ => SlabType::Top,
                },
            };

            slab_props.to_state_id(args.block)
        })
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let slab_props = SlabProperties::from_state_id(args.state_id, args.block);

        slab_props.r#type
            == match args.direction {
                BlockDirection::Up => SlabType::Bottom,
                BlockDirection::Down => SlabType::Top,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0..0.5 => SlabType::Top,
                    _ => SlabType::Bottom,
                },
            }
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // No tag gate needed: the oxidation_stages table below only contains the
            // cut copper slab family, so this is a no-op for every other slab type.

            let current_state_id = args.world.get_block_state_id(args.position);
            let slab_props = SlabProperties::from_state_id(current_state_id, args.block);

            let oxidation_stages = [
                &Block::CUT_COPPER_SLAB,
                &Block::EXPOSED_CUT_COPPER_SLAB,
                &Block::WEATHERED_CUT_COPPER_SLAB,
                &Block::OXIDIZED_CUT_COPPER_SLAB,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props = SlabProperties::default(next_block);
                    new_props.r#type = slab_props.r#type;
                    new_props.waterlogged = slab_props.waterlogged;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}
