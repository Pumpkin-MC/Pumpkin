use pumpkin_data::block_properties::{HorizontalFacing, ShelfMushroomLikeProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, FacingExt, HorizontalFacingExt, Mirror,
    Rotation,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnLandedUponArgs,
    OnPlaceArgs, UpdateEntityMovementAfterFallOnArgs, bounce_entity_after_fall,
};
use crate::entity::EntityBase;

const MAX_AGE: u8 = 1;

type ShelfMushroomProperties = ShelfMushroomLikeProperties;

#[pumpkin_block("minecraft:shelf_mushroom")]
pub struct ShelfMushroomBlock;

impl ShelfMushroomBlock {
    fn can_survive(world: &dyn BlockAccessor, pos: &BlockPos, facing: HorizontalFacing) -> bool {
        let support_pos = pos.offset(facing.to_offset());
        world
            .get_block_state(&support_pos)
            .is_center_solid(facing.opposite().to_block_direction())
    }
}

impl BlockBehaviour for ShelfMushroomBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let state_id = args.block_accessor.get_block_state_id(args.position);
        if state_id != Block::AIR.default_state.id {
            let props = ShelfMushroomProperties::from_state_id(state_id);
            return Self::can_survive(args.block_accessor, args.position, props.facing);
        }
        for facing in [
            HorizontalFacing::North,
            HorizontalFacing::South,
            HorizontalFacing::West,
            HorizontalFacing::East,
        ] {
            if Self::can_survive(args.block_accessor, args.position, facing) {
                return true;
            }
        }
        false
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = ShelfMushroomProperties::default(args.block);
        props.age = 0;

        if args.direction != BlockDirection::Up
            && args.direction != BlockDirection::Down
            && let Some(facing) = args.direction.to_horizontal_facing()
            && Self::can_survive(args.world, args.position, facing)
        {
            props.facing = facing;
            return props.to_state_id(args.block);
        }

        let directions = args.player.get_entity().get_entity_facing_order();
        for dir in directions {
            if let Some(facing) = dir.to_horizontal_facing()
                && Self::can_survive(args.world, args.position, facing)
            {
                props.facing = facing;
                return props.to_state_id(args.block);
            }
        }

        Block::AIR.default_state.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = ShelfMushroomProperties::from_state_id(args.state_id);
        if args.direction == props.facing.to_block_direction()
            && !Self::can_survive(args.world, args.position, props.facing)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        ShelfMushroomProperties::from_state_id(args.state_id).age < MAX_AGE
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let mut props = ShelfMushroomProperties::from_state_id(args.state_id);
        if props.age < MAX_AGE {
            props.age += 1;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 0.0);
        }
    }

    fn update_entity_movement_after_fall_on(&self, args: UpdateEntityMovementAfterFallOnArgs<'_>) {
        bounce_entity_after_fall(args.entity, 0.8);
        if args.entity.get_living_entity().is_some() {
            let entity = args.entity.get_entity();
            entity.world.load().play_sound(
                Sound::BlockShelfMushroomBounce,
                SoundCategory::Blocks,
                &entity.pos.load(),
            );
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut props = ShelfMushroomProperties::from_state_id(state_id);
        props.facing = rotation.rotate_horizontal(props.facing);
        BlockState::from_id(props.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let mut props = ShelfMushroomProperties::from_state_id(state_id);
        props.facing = mirror.mirror_horizontal(props.facing);
        BlockState::from_id(props.to_state_id(block))
    }
}
