use pumpkin_data::{
    Block,
    block_properties::{
        BlockProperties, EnumVariants, Integer0To7, WallTorchLikeProperties, WheatLikeProperties,
    },
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

type AttachedStemProperties = WallTorchLikeProperties;

type StemProperties = WheatLikeProperties;
pub struct AttachedStemBlock;

impl BlockMetadata for AttachedStemBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            Block::ATTACHED_PUMPKIN_STEM.name,
            Block::ATTACHED_MELON_STEM.name,
        ]
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
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
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
                props.age = Integer0To7::from_index(7);
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
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block == &Block::FARMLAND
    }
}
