use crate::entity::EntityBase;
use pumpkin_data::{
    Block, BlockDirection, BlockStateId, FacingExt, HorizontalFacingExt,
    block_properties::{AttachFace, HorizontalFacing},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::{
    block::{BlockIsReplacing, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs},
    entity::player::Player,
};

pub trait WallMountedBlock: Send + Sync {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection;

    fn get_placement_face(
        &self,
        player: &Player,
        direction: BlockDirection,
    ) -> (AttachFace, HorizontalFacing) {
        let face = match direction {
            BlockDirection::Up => AttachFace::Ceiling,
            BlockDirection::Down => AttachFace::Floor,
            _ => AttachFace::Wall,
        };

        let facing = if direction == BlockDirection::Up || direction == BlockDirection::Down {
            player.get_entity().get_horizontal_facing()
        } else {
            direction.opposite().to_cardinal_direction()
        };

        (face, facing)
    }

    fn get_placement_face_for_args(
        &self,
        args: &OnPlaceArgs<'_>,
    ) -> (AttachFace, HorizontalFacing) {
        let mut directions = args.player.get_entity().get_entity_facing_order();

        if args.replacing == BlockIsReplacing::None {
            let face = args.direction.to_facing();
            let mut i = 0;
            while i < directions.len() && directions[i] != face {
                i += 1;
            }

            if i > 0 {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        }

        for dir in directions {
            let direction = dir.to_block_direction();
            if self.can_place_at(args.world, args.position, direction) {
                return self.get_placement_face(args.player, direction);
            }
        }

        self.get_placement_face(args.player, args.direction)
    }

    fn can_place_at_args(&self, args: &CanPlaceAtArgs<'_>) -> bool {
        if args.player.is_some() {
            return BlockDirection::all()
                .iter()
                .any(|&dir| self.can_place_at(args.block_accessor, args.position, dir));
        }

        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block).opposite());

        self.can_place_at(args.block_accessor, args.position, direction)
    }

    /// Gets the direction to check for placement validation based on clicked face
    /// This returns the `BlockDirection` that should have a solid surface for placement
    fn get_placement_direction(
        &self,
        player: &Player,
        direction: BlockDirection,
    ) -> BlockDirection {
        let (face, facing) = self.get_placement_face(player, direction);
        match face {
            AttachFace::Floor => BlockDirection::Up,
            AttachFace::Ceiling => BlockDirection::Down,
            AttachFace::Wall => facing.to_block_direction(),
        }
    }

    fn can_place_at(
        &self,
        world: &dyn BlockAccessor,
        pos: &BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let block_pos = pos.offset(direction.to_offset());
        let block_state = world.get_block_state(&block_pos);
        block_state.is_side_solid(direction.opposite())
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if self.get_direction(args.state_id, args.block).opposite() == args.direction
            && !self.can_place_at(args.world, args.position, args.direction)
        {
            Block::AIR.default_state.id
        } else {
            args.state_id
        }
    }
}
