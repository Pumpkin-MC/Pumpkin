use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::{BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs};

use super::FireBlockBase;
use crate::block::OnEntityCollisionArgs;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    #[must_use]
    pub fn is_soul_base(block: &Block) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_SOUL_FIRE_BASE_BLOCKS)
    }

    #[must_use]
    pub fn can_survive_at(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let down = pos.down();
        let (block, state) = block_accessor.get_block_and_state(&down);
        Self::is_soul_base(block) && state.is_side_solid(BlockDirection::Up)
    }
}

impl BlockBehaviour for SoulFireBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        FireBlockBase::apply_fire_collision(&args, true);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::can_survive_at(args.world, args.position) {
            return Block::AIR.default_state.id;
        }

        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive_at(args.block_accessor, args.position)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        {
            FireBlockBase::broken(args.world, *args.position);
        }
    }
}
