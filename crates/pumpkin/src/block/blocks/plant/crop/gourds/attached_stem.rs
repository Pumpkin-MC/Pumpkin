use pumpkin_data::{
    Block, BlockId, BlockStateId,
    block_properties::{BlockProperties, WallTorchLikeProperties, WheatLikeProperties},
    tag::{self, Taggable},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

type AttachedStemProperties = WallTorchLikeProperties;

type StemProperties = WheatLikeProperties;
pub struct AttachedStemBlock;

impl BlockMetadata for AttachedStemBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::ATTACHED_PUMPKIN_STEM, BlockId::ATTACHED_MELON_STEM].into()
    }
}

impl AttachedStemBlock {
    fn get_stem(block: &Block) -> &Block {
        match block.id {
            id if id == Block::ATTACHED_PUMPKIN_STEM.id => &Block::PUMPKIN_STEM,
            id if id == Block::ATTACHED_MELON_STEM.id => &Block::MELON_STEM,
            _ => &Block::MELON_STEM, // Should never happen
        }
    }

    fn get_gourd(block: &Block) -> &Block {
        match block.id {
            id if id == Block::ATTACHED_PUMPKIN_STEM.id => &Block::PUMPKIN,
            id if id == Block::ATTACHED_MELON_STEM.id => &Block::MELON,
            _ => &Block::MELON, // Should never happen
        }
    }
}

impl BlockBehaviour for AttachedStemBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = AttachedStemProperties::from_state_id(args.state_id, args.block);
            if args.direction.to_horizontal_facing() == Some(props.facing)
                && args.neighbor_state_id != Self::get_gourd(args.block).default_state.id
            {
                let mut props = StemProperties::default(Self::get_stem(args.block));
                props.age = 7;
                return props.to_state_id(Self::get_stem(args.block));
            }
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for AttachedStemBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos);
        if block == &Block::ATTACHED_PUMPKIN_STEM {
            block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_PUMPKIN_STEM)
        } else {
            block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_MELON_STEM)
        }
    }
}
