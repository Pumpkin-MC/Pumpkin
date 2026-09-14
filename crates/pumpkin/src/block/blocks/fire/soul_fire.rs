use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;

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
}

impl BlockBehaviour for SoulFireBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        FireBlockBase::apply_fire_collision(&args, true);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::is_soul_base(args.world.get_block(&args.position.down())) {
            return Block::AIR.default_state.id;
        }

        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::is_soul_base(args.block_accessor.get_block(&args.position.down()))
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        {
            FireBlockBase::broken(args.world, *args.position);
        }
    }
}
