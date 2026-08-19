use crate::block::BlockFuture;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;

type GlassPaneProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::BlockBehaviour;
use crate::world::World;

#[pumpkin_block_from_tag("c:glass_panes")]
pub struct GlassPaneBlock;

impl BlockBehaviour for GlassPaneBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut pane_props = GlassPaneProperties::default(args.block);
            pane_props.waterlogged = args.replacing.water_source();

            compute_pane_state(pane_props, args.world, args.block, args.position)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let pane_props = GlassPaneProperties::from_state_id(args.state_id, args.block);
            compute_pane_state(pane_props, args.world, args.block, args.position)
        })
    }
}

pub fn compute_pane_state(
    mut pane_props: GlassPaneProperties,
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
            || other_block == &Block::IRON_BARS
            || other_block.has_tag(&tag::Block::MINECRAFT_WALLS);

        match direction {
            HorizontalFacing::North => pane_props.north = connected,
            HorizontalFacing::South => pane_props.south = connected,
            HorizontalFacing::West => pane_props.west = connected,
            HorizontalFacing::East => pane_props.east = connected,
        }
    }

    pane_props.to_state_id(block)
}
