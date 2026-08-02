use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, RandomTickArgs};
use crate::world::World;

/// `LightEngine#getLightDampeningInto` result at which a covering block blocks all light.
const MAX_LIGHT_DAMPENING: u8 = 15;

/// `NyliumBlock`: crimson and warped nylium die back into netherrack once they no longer have
/// enough light above them.
///
/// Unlike `GrassBlock`, vanilla's `NyliumBlock#randomTick` has no spreading branch at all —
/// nylium only spreads onto adjacent netherrack via bonemeal (`BonemealableBlock#performBonemeal`,
/// `NEIGHBOR_SPREADER` type), never on its own from a random tick. `performBonemeal` places
/// nether vegetation configured features and is out of scope here; only the die-back half of the
/// block is implemented.
#[pumpkin_block_from_tag("minecraft:nylium")]
pub struct NyliumBlock;

impl BlockBehaviour for NyliumBlock {
    /// `NyliumBlock#randomTick`: revert to netherrack once the block above dampens all light.
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_be_nylium(args.world, args.position) {
                args.world
                    .set_block_state(
                        args.position,
                        Block::NETHERRACK.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}

/// `NyliumBlock#canBeNylium`: `LightEngine.getLightDampeningInto(state, aboveState, Direction.UP,
/// aboveState.getLightDampening()) < 15`. Pumpkin has no face-occlusion lookup, so — as in
/// `grass_block.rs`'s `can_be_grass` — the raw opacity of the covering block stands in for the
/// dampened-light calculation. This is a known divergence for blocks that occlude downwards but
/// carry low opacity (carpets, bottom slabs, thin snow); the common fully-opaque-ceiling case
/// still kills the nylium correctly.
fn can_be_nylium(world: &World, position: &BlockPos) -> bool {
    let above_state = world.get_block_state(&position.up());
    above_state.opacity < MAX_LIGHT_DAMPENING
}
