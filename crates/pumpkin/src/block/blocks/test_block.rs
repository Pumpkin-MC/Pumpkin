use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{GameMode, PermissionLvl};

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::test_block::{TestBlockBlockEntity, TestBlockMode};
use crate::block::entities::test_instance_block::TestInstanceBlockBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetRedstonePowerArgs, NormalUseArgs,
    OnNeighborUpdateArgs, OnScheduledTickArgs, PlacedArgs,
};

#[pumpkin_block("minecraft:test_block")]
pub struct TestBlock;

impl BlockBehaviour for TestBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.permission_lvl.load() < PermissionLvl::Two {
            return BlockActionResult::Pass;
        }
        if args.player.gamemode.load() != GameMode::Creative {
            return BlockActionResult::Pass;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        args.world.update_block_entity(&block_entity);
        BlockActionResult::SuccessServer
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let mode = TestBlockMode::from_block_state(args.state_id).unwrap_or(TestBlockMode::Start);
        let entity = TestBlockBlockEntity::new_with_mode(*args.position, mode);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return;
        };

        // Vanilla TestBlock.neighborChanged treats START as output-only. The mode
        // comes from the block state; the block entity only carries runtime state.
        let mode = TestBlockMode::from_block_state(args.world.get_block_state_id(args.position))
            .unwrap_or_else(|| test_block.mode());
        if mode == TestBlockMode::Start {
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
        if args.block != &Block::TEST_BLOCK
            || TestBlockMode::from_block_state(args.state.id) != Some(TestBlockMode::Start)
        {
            return 0;
        }

        let Some(entity) = args.world.get_block_entity(args.position) else {
            return 0;
        };
        let Some(test_block) = entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return 0;
        };

        if test_block.is_powered() { 15 } else { 0 }
    }
}

#[pumpkin_block("minecraft:test_instance_block")]
pub struct TestInstanceBlock;

impl BlockBehaviour for TestInstanceBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.permission_lvl.load() < PermissionLvl::Two {
            return BlockActionResult::Pass;
        }
        if args.player.gamemode.load() != GameMode::Creative {
            return BlockActionResult::Pass;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::Pass;
        };
        args.world.update_block_entity(&block_entity);
        BlockActionResult::SuccessServer
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = TestInstanceBlockBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }
}
