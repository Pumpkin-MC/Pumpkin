use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{AttackArgs, BlockBehaviour, BlockFuture, NormalUseArgs, PlacedArgs};
use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use rand::{RngExt, rng};
use std::sync::Arc;

#[pumpkin_block("minecraft:dragon_egg")]
pub struct DragonEggBlock;

impl DragonEggBlock {
    async fn teleport(&self, world: &Arc<World>, pos: &BlockPos, state_id: BlockStateId) {
        for _ in 0..1000 {
            let x = pos.0.x + rng().random_range(0..16) - rng().random_range(0..16);
            let y = pos.0.y + rng().random_range(0..8) - rng().random_range(0..8);
            let z = pos.0.z + rng().random_range(0..16) - rng().random_range(0..16);
            let test_pos = BlockPos::new(x, y, z);

            let state = world.get_block_state(&test_pos);
            let below_state = world.get_block_state(&test_pos.down());

            let in_world_border = world
                .worldborder
                .lock()
                .await
                .contains_block(test_pos.0.x, test_pos.0.z);
            let in_build_height = test_pos.0.y >= world.dimension.min_y
                && test_pos.0.y < world.dimension.min_y + world.dimension.height;

            if state.is_air() && !below_state.is_air() && in_world_border && in_build_height {
                // Re-read all three states after the async border check. The Java method is
                // synchronous; this closes the equivalent async window before the write.
                let current_source = world.get_block_state(pos);
                let current_target = world.get_block_state(&test_pos);
                let current_below = world.get_block_state(&test_pos.down());
                if current_source.id != state_id
                    || !current_target.is_air()
                    || current_below.is_air()
                {
                    continue;
                }
                world
                    .set_block_state(
                        &test_pos,
                        state_id,
                        pumpkin_world::world::BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
                if world.get_block_state(pos).id != state_id {
                    return;
                }
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
    fn attack<'a>(&'a self, args: AttackArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.teleport(args.world, args.position, args.state.id)
                .await;
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 5, TickPriority::Normal);
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state(args.position).id;
            self.teleport(args.world, args.position, state_id).await;
            BlockActionResult::Success
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
