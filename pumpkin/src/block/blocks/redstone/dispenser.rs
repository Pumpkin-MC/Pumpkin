use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, BlockFuture, OnNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::block_properties::{BlockProperties, DispenserLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;

#[pumpkin_block("minecraft:dispenser")]
pub struct DispenserBlock;

impl BlockBehaviour for DispenserBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = DispenserLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Quasi-connectivity: dispensers check power at their own position AND one block above,
            // matching vanilla behavior. This is the same "bug" that pistons and droppers use.
            let powered = block_receives_redstone_power(args.world, args.position).await
                || block_receives_redstone_power(args.world, &args.position.up()).await;
            let mut props = DispenserLikeProperties::from_state_id(
                args.world.get_block_state(args.position).await.id,
                args.block,
            );
            if powered && !props.triggered {
                args.world
                    .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal)
                    .await;
                props.triggered = true;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            } else if !powered && props.triggered {
                props.triggered = false;
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
