use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs};
use crate::entity::EntityBase;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_world::world::BlockFlags;

type SkeletonSkullLikeProperties = pumpkin_data::block_properties::SkeletonSkullLikeProperties;

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

use crate::block::PlacedArgs;
use crate::block::entities::skull::SkullBlockEntity;
use std::sync::Arc;

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
            let mut props = SkeletonSkullLikeProperties::from_state_id(state.id, args.block);
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
}
