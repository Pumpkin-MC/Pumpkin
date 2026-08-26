use pumpkin_data::block_properties::{BlockProperties, CocoaLikeProperties, HorizontalFacing};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{
    Block, BlockState, BlockStateId, FacingExt, HorizontalFacingExt, Mirror, Rotation, tag,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    RandomTickArgs,
};
use crate::entity::EntityBase;

pub const MAX_AGE: u8 = 2;

type CocoaProperties = CocoaLikeProperties;

#[pumpkin_block("minecraft:cocoa")]
pub struct CocoaBlock;

impl CocoaBlock {
    #[must_use]
    pub fn can_survive(
        world: &dyn BlockAccessor,
        pos: &BlockPos,
        facing: HorizontalFacing,
    ) -> bool {
        let support_pos = pos.offset(facing.to_offset());
        let block = world.get_block(&support_pos);
        block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_COCOA)
    }
}

impl BlockBehaviour for CocoaBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let state_id = args.block_accessor.get_block_state_id(args.position);
        if state_id != Block::AIR.default_state.id {
            let props = CocoaProperties::from_state_id(state_id, args.block);
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
        let mut props = CocoaProperties::default(args.block);
        props.age = 0;

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
        let props = CocoaProperties::from_state_id(args.state_id, args.block);
        if args.direction == props.facing.to_block_direction()
            && !Self::can_survive(args.world, args.position, props.facing)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::random::<u8>().is_multiple_of(5) {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CocoaProperties::from_state_id(state_id, args.block);
            if props.age < MAX_AGE {
                props.age += 1;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let props = CocoaProperties::from_state_id(args.state_id, args.block);
        props.age < MAX_AGE
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        {
            let mut props = CocoaProperties::from_state_id(args.state_id, args.block);
            if props.age < MAX_AGE {
                props.age += 1;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut props = CocoaProperties::from_state_id(state_id, block);
        props.facing = rotation.rotate_horizontal(props.facing);
        BlockState::from_id(props.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let mut props = CocoaProperties::from_state_id(state_id, block);
        props.facing = mirror.mirror_horizontal(props.facing);
        BlockState::from_id(props.to_state_id(block))
    }
}
