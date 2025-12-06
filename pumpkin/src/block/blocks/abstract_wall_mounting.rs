use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockFace, HorizontalFacing},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::{
    block::{BlockFuture, GetStateForNeighborUpdateArgs},
    entity::player::Player,
};

pub trait WallMountedBlock: Send + Sync {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection;

    fn get_placement_face(
        &self,
        player: &Player,
        direction: BlockDirection,
    ) -> (BlockFace, HorizontalFacing) {
        let face = match direction {
            BlockDirection::Up => BlockFace::Ceiling,
            BlockDirection::Down => BlockFace::Floor,
            _ => BlockFace::Wall,
        };

        let facing = if direction == BlockDirection::Up || direction == BlockDirection::Down {
            player.living_entity.entity.get_horizontal_facing()
        } else {
            direction.opposite().to_cardinal_direction()
        };

        (face, facing)
    }

    fn can_place_at<'a>(
        &'a self,
        world: &'a dyn BlockAccessor,
        pos: &'a BlockPos,
        direction: BlockDirection,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block_pos = pos.offset(direction.to_offset());
            let block_state = world.get_block_state(&block_pos).await;
            block_state.is_side_solid(direction.opposite())
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if self.get_direction(args.state_id, args.block).opposite() == args.direction
                && !self
                    .can_place_at(args.world, args.position, args.direction)
                    .await
            {
                Block::AIR.default_state.id
            } else {
                args.state_id
            }
        })
    }
}
