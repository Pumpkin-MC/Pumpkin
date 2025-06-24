use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockFace, HorizontalFacing},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::entity::player::Player;

#[async_trait]
pub trait WallMountedBlock {
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

    async fn can_place_at(
        &self,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let block_pos = pos.offset(direction.to_offset());
        let block_state = world.get_block_state(&block_pos).await;
        block_state.is_side_solid(direction.opposite())
    }

    async fn get_state_for_neighbor_update(
        &self,
        state_id: BlockStateId,
        block: &Block,
        direction: BlockDirection,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> Option<BlockStateId> {
        (self.get_direction(state_id, block).opposite() == direction
            && !self.can_place_at(world, pos, direction).await)
            .then(|| Block::AIR.default_state.id)
    }
}
