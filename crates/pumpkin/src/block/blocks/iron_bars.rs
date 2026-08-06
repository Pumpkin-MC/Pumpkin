use crate::block::BlockFuture;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::RandomTickArgs;
use crate::block::blocks::copper_weathering;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;

type IronBarsProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::BlockBehaviour;
use crate::world::World;

// Covers the whole `minecraft:bars` tag: iron_bars plus the copper_bars oxidation
// family (unwaxed and waxed). Weathering below only fires for the copper members since
// their oxidation_stages table doesn't include iron_bars.
#[pumpkin_block_from_tag("minecraft:bars")]
pub struct IronBarsBlock;

impl BlockBehaviour for IronBarsBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut bars_props = IronBarsProperties::default(args.block);
            bars_props.waterlogged = args.replacing.water_source();

            compute_bars_state(bars_props, args.world, args.block, args.position)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let bars_props = IronBarsProperties::from_state_id(args.state_id, args.block);
            compute_bars_state(bars_props, args.world, args.block, args.position)
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // No tag gate needed: the oxidation_stages table below only contains the
            // copper_bars family, so this is a no-op for iron_bars.
            let current_state_id = args.world.get_block_state_id(args.position);
            let current_props = IronBarsProperties::from_state_id(current_state_id, args.block);

            let oxidation_stages = [
                &Block::COPPER_BARS,
                &Block::EXPOSED_COPPER_BARS,
                &Block::WEATHERED_COPPER_BARS,
                &Block::OXIDIZED_COPPER_BARS,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props = IronBarsProperties::default(next_block);
                    new_props.waterlogged = current_props.waterlogged;
                    new_props.north = current_props.north;
                    new_props.south = current_props.south;
                    new_props.east = current_props.east;
                    new_props.west = current_props.west;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}

pub fn compute_bars_state(
    mut bars_props: IronBarsProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> BlockStateId {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) = world.get_block_and_state(&other_block_pos);

        let connected = other_block == block
            || other_block_state.is_side_solid(direction.opposite().to_block_direction())
            || other_block.has_tag(&tag::Block::C_GLASS_PANES)
            || other_block.has_tag(&tag::Block::MINECRAFT_WALLS);

        match direction {
            HorizontalFacing::North => bars_props.north = connected,
            HorizontalFacing::South => bars_props.south = connected,
            HorizontalFacing::West => bars_props.west = connected,
            HorizontalFacing::East => bars_props.east = connected,
        }
    }

    bars_props.to_state_id(block)
}
