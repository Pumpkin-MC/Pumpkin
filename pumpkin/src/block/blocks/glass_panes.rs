use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

type GlassPaneProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;

pub struct GlassPaneBlock;
impl BlockMetadata for GlassPaneBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:glass_panes").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for GlassPaneBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut pane_props = GlassPaneProperties::default(args.block);
        pane_props.waterlogged = args.replacing.water_source();

        compute_pane_state(pane_props, args.world, args.block, args.location).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let pane_props = GlassPaneProperties::from_state_id(args.state_id, args.block);
        compute_pane_state(pane_props, args.world, args.block, args.location).await
    }
}

pub async fn compute_pane_state(
    mut pane_props: GlassPaneProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) =
            world.get_block_and_block_state(&other_block_pos).await;

        let connected = other_block == block
            || other_block_state.is_side_solid(direction.opposite())
            || other_block.is_tagged_with("c:glass_panes").unwrap()
            || other_block == &Block::IRON_BARS
            || other_block.is_tagged_with("minecraft:walls").unwrap();

        match direction {
            BlockDirection::North => pane_props.north = connected,
            BlockDirection::South => pane_props.south = connected,
            BlockDirection::West => pane_props.west = connected,
            BlockDirection::East => pane_props.east = connected,
            _ => {}
        }
    }

    pane_props.to_state_id(block)
}
