use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, BonemealArgs};

#[pumpkin_block("minecraft:rooted_dirt")]
pub struct RootedDirtBlock;

impl BlockBehaviour for RootedDirtBlock {
    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let below = args.position.down();
        args.world.is_in_height_limit(below.0.y)
            && args.world.is_loaded(&below)
            && args.world.get_block_state(&below).is_air()
    }

    fn perform_bonemeal<'a>(&'a self, args: BonemealArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .set_block_state(
                    &args.position.down(),
                    Block::HANGING_ROOTS.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        })
    }
}
