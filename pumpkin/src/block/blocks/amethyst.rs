use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::BlockMetadata;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::FacingExt;
use pumpkin_data::block_properties::AmethystClusterLikeProperties;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_world::BlockStateId;

pub struct AmethystBlock;

impl BlockMetadata for AmethystBlock {
    fn ids() -> Box<[u16]> {
        [
            Block::SMALL_AMETHYST_BUD.id,
            Block::MEDIUM_AMETHYST_BUD.id,
            Block::LARGE_AMETHYST_BUD.id,
            Block::AMETHYST_CLUSTER.id,
        ]
        .into()
    }
}

impl BlockBehaviour for AmethystBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = AmethystClusterLikeProperties::from_state_id(
                args.block.default_state.id,
                args.block,
            );
            props.facing = args.direction.to_facing().opposite();
            props.waterlogged = args.replacing.water_source();
            props.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Use the provided direction, or fallback to the current state's direction if missing
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block));

        WallMountedBlock::can_place_at(self, args.block_accessor, args.position, direction)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { WallMountedBlock::get_state_for_neighbor_update(self, args).await })
    }
}

impl WallMountedBlock for AmethystBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = AmethystClusterLikeProperties::from_state_id(state_id, block);
        props.facing.to_block_direction()
    }
}
