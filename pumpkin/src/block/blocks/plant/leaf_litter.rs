use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, HorizontalFacing, Integer1To4},
};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

type LeafLitterProperties = pumpkin_data::block_properties::LeafLitterLikeProperties;

pub struct LeafLitterBlock;

impl BlockMetadata for LeafLitterBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["leaf_litter"]
    }
}

#[async_trait]
impl PumpkinBlock for LeafLitterBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let block_below = block_accessor.get_block_state(&block_pos.down()).await;
        block_below.is_side_solid(BlockDirection::Up)
    }

    async fn can_update_at(
        &self,
        _world: &World,
        block: &Block,
        state_id: BlockStateId,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
    ) -> bool {
        // Allow placing multiple leaf litter in same position
        let current_props = LeafLitterProperties::from_state_id(state_id, block);
        can_add_segment(&current_props)
    }

    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        match replacing {
            BlockIsReplacing::Itself(existing_state_id) => {
                let mut props = LeafLitterProperties::from_state_id(existing_state_id, block);

                if can_add_segment(&props) {
                    props.segment_amount = get_next_segment_amount(props.segment_amount);
                    props.to_state_id(block)
                } else {
                    existing_state_id
                }
            }
            _ => {
                // Set first segment orientation based on player direction
                let player_facing = player.living_entity.entity.get_horizontal_facing();
                let mut props = LeafLitterProperties::default(block);
                props.segment_amount = Integer1To4::L1;
                props.facing = get_facing_for_segment(player_facing, Integer1To4::L1);
                props.to_state_id(block)
            }
        }
    }
}

fn can_add_segment(props: &LeafLitterProperties) -> bool {
    matches!(
        props.segment_amount,
        Integer1To4::L1 | Integer1To4::L2 | Integer1To4::L3
    )
}

fn get_next_segment_amount(current: Integer1To4) -> Integer1To4 {
    match current {
        Integer1To4::L1 => Integer1To4::L2,
        Integer1To4::L2 => Integer1To4::L3,
        Integer1To4::L3 => Integer1To4::L4,
        Integer1To4::L4 => Integer1To4::L4,
    }
}

/// Minecraft leaf litter quadrant sequence: bottom right -> top right -> top left -> bottom left
fn get_facing_for_segment(
    player_facing: HorizontalFacing,
    segment_amount: Integer1To4,
) -> HorizontalFacing {
    match (player_facing, segment_amount) {
        (HorizontalFacing::North, Integer1To4::L1) => HorizontalFacing::South, // bottom right
        (HorizontalFacing::North, Integer1To4::L2) => HorizontalFacing::East,  // top right
        (HorizontalFacing::North, Integer1To4::L3) => HorizontalFacing::North, // top left
        (HorizontalFacing::North, Integer1To4::L4) => HorizontalFacing::West,  // bottom left

        (HorizontalFacing::South, Integer1To4::L1) => HorizontalFacing::North, // bottom right
        (HorizontalFacing::South, Integer1To4::L2) => HorizontalFacing::West,  // top right
        (HorizontalFacing::South, Integer1To4::L3) => HorizontalFacing::South, // top left
        (HorizontalFacing::South, Integer1To4::L4) => HorizontalFacing::East,  // bottom left

        (HorizontalFacing::East, Integer1To4::L1) => HorizontalFacing::West, // bottom right
        (HorizontalFacing::East, Integer1To4::L2) => HorizontalFacing::South, // top right
        (HorizontalFacing::East, Integer1To4::L3) => HorizontalFacing::East, // top left
        (HorizontalFacing::East, Integer1To4::L4) => HorizontalFacing::North, // bottom left

        (HorizontalFacing::West, Integer1To4::L1) => HorizontalFacing::East, // bottom right
        (HorizontalFacing::West, Integer1To4::L2) => HorizontalFacing::North, // top right
        (HorizontalFacing::West, Integer1To4::L3) => HorizontalFacing::West, // top left
        (HorizontalFacing::West, Integer1To4::L4) => HorizontalFacing::South, // bottom left
    }
}
