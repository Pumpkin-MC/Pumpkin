use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, BrokenArgs, NormalUseArgs, PlacedArgs};
use crate::world::World;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use rand::{RngExt, rng};
use std::sync::Arc;

#[pumpkin_block("minecraft:dragon_egg")]
pub struct DragonEggBlock;

impl DragonEggBlock {
    async fn teleport(&self, world: &Arc<World>, pos: &BlockPos) {
        for _ in 0..1000 {
            let x = pos.0.x + rng().random_range(-16..16);
            let y = pos.0.y + rng().random_range(-8..8);
            let z = pos.0.z + rng().random_range(-16..16);
            let test_pos = BlockPos::new(x, y, z);

            let state = world.get_block_state(&test_pos).await;
            let below_state = world.get_block_state(&test_pos.down()).await;

            if state.is_air() && !below_state.is_air() {
                let current_state = world.get_block_state(pos).await;
                world
                    .set_block_state(
                        &test_pos,
                        current_state.id,
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                world
                    .set_block_state(
                        pos,
                        pumpkin_data::Block::AIR.default_state.id,
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                return;
            }
        }
    }
}

impl BlockBehaviour for DragonEggBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 5, TickPriority::Normal)
                .await;
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            self.teleport(args.world, args.position).await;
            BlockActionResult::Success
        })
    }

    // Dragon egg is typically teleported when attacked
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.teleport(args.world, args.position).await;
        })
    }

    fn on_scheduled_tick<'a>(
        &'a self,
        args: crate::block::OnScheduledTickArgs<'a>,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FallingBlock::on_scheduled_tick(&FallingBlock, args).await;
        })
    }
}
