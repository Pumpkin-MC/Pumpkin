use pumpkin_data::{Block, BlockId};

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::test_block::{TestBlockBlockEntity, TestBlockMode};
use crate::block::{
    BlockBehaviour, BlockMetadata, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    OnNeighborUpdateArgs, OnScheduledTickArgs,
};

pub struct TestBlock;

impl BlockMetadata for TestBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::TEST_BLOCK].into()
    }
}

impl BlockBehaviour for TestBlock {
    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return;
        };

        // TestBlock.neighborChanged: START is an output-only test block. Every
        // other mode triggers exactly once on the rising edge and only rearms
        // after the incoming signal falls again.
        if test_block.mode() == TestBlockMode::Start {
            return;
        }

        let should_trigger = block_receives_redstone_power(args.world, args.position);
        let is_powered = test_block.is_powered();
        if should_trigger && !is_powered {
            test_block.set_powered(true);
            test_block.trigger(args.world);
        } else if !should_trigger && is_powered {
            test_block.set_powered(false);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return;
        };
        test_block.reset(args.world);
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.block != &Block::TEST_BLOCK {
            return 0;
        }
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return 0;
        };
        let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return 0;
        };

        if test_block.mode() == TestBlockMode::Start && test_block.is_powered() {
            15
        } else {
            0
        }
    }
}
