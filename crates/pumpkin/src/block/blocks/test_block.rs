use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{BlockProperties, TestBlockLikeProperties, TestBlockMode};

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::test_block::TestBlockBlockEntity;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    OnNeighborUpdateArgs, OnScheduledTickArgs,
};

/// `net.minecraft.world.level.block.TestBlock`.
pub struct TestBlock;

impl BlockMetadata for TestBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::TEST_BLOCK].into()
    }
}

impl BlockBehaviour for TestBlock {
    /// `TestBlock.tick`: the scheduled tick resets the block entity.
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(entity) = args.world.get_block_entity(args.position)
                && let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>()
            {
                test_block.reset(args.world).await;
            }
        })
    }

    /// `TestBlock.neighborChanged`: a rising redstone edge triggers non-START blocks; a
    /// falling edge only clears `powered`. START-mode blocks are driven by `trigger()`
    /// from the test framework instead, so they ignore neighbour signal entirely.
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let Some(entity) = args.world.get_block_entity(args.position) else {
                return;
            };
            let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
                return;
            };
            if test_block.get_mode().await == TestBlockMode::Start {
                return;
            }
            let should_trigger = block_receives_redstone_power(args.world, args.position).await;
            let is_powered = test_block.is_powered();
            if should_trigger && !is_powered {
                test_block.set_powered(true);
                test_block.trigger(args.world).await;
            } else if !should_trigger && is_powered {
                test_block.set_powered(false);
            }
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    /// `TestBlock.ownSignal`: only a powered START block emits, and it emits full strength.
    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = TestBlockLikeProperties::from_state_id(args.state.id, args.block);
            if props.mode != TestBlockMode::Start {
                return 0;
            }
            let powered = args
                .world
                .get_block_entity(args.position)
                .is_some_and(|entity| {
                    entity
                        .as_any()
                        .downcast_ref::<TestBlockBlockEntity>()
                        .is_some_and(TestBlockBlockEntity::is_powered)
                });
            if powered { 15 } else { 0 }
        })
    }
}
