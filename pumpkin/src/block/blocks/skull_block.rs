use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

type SkeletonSkullLikeProperties = pumpkin_data::block_properties::SkeletonSkullLikeProperties;

pub struct SkullBlock;

impl BlockMetadata for SkullBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            "skeleton_skull",
            "wither_skeleton_skull",
            "player_head",
            "zombie_head",
            "creeper_head",
            "piglin_head",
            "dragon_head",
        ]
    }
}

impl BlockBehaviour for SkullBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = SkeletonSkullLikeProperties::default(args.block);
            props.rotation = args.player.get_entity().get_rotation_16();
            props.powered = block_receives_redstone_power(args.world, args.position).await;
            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut props = SkeletonSkullLikeProperties::from_state_id(state.id, args.block);
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;
            if props.powered != is_receiving_power {
                props.powered = is_receiving_power;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }
}
