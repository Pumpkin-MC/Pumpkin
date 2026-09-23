use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::skull::SkullBlockEntity;
use crate::block::{
    BlockBehaviour, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
    PlacedArgs,
};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::SkeletonSkullLikeProperties;
use pumpkin_data::{BlockId, BlockState, BlockStateId};
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

pub struct SkullBlock;

impl BlockMetadata for SkullBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::SKELETON_SKULL,
            BlockId::PLAYER_HEAD,
            BlockId::ZOMBIE_HEAD,
            BlockId::CREEPER_HEAD,
            BlockId::PIGLIN_HEAD,
            BlockId::DRAGON_HEAD,
        ]
        .into()
    }
}

impl BlockBehaviour for SkullBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = SkullBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SkeletonSkullLikeProperties::default(args.block);
        props.rotation = args.player.get_entity().get_rotation_16();
        props.powered = block_receives_redstone_power(args.world, args.position);
        props.to_state_id(args.block)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state = args.world.get_block_state(args.position);
            let mut props = SkeletonSkullLikeProperties::from_state_id(state.id);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position);
            if props.powered != is_receiving_power {
                props.powered = is_receiving_power;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                );
            }
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
