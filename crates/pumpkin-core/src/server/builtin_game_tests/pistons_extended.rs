use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{Facing, RedstoneOreLikeProperties, StickyPistonLikeProperties},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

fn piston_state(block: &Block, facing: Facing) -> BlockStateId {
    let mut props = StickyPistonLikeProperties::default(block);
    props.facing = facing;
    props.extended = false;
    props.to_state_id(block)
}

pub(super) async fn test_piston_pushes_upward_and_retracts(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let piston = BlockPos::new(1, 1, 1);
    let cargo = BlockPos::new(1, 2, 1);
    let dest = BlockPos::new(1, 3, 1);
    let power = BlockPos::new(2, 1, 1);

    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::Up))
        .await?;
    helper.set_block_default(&cargo, &Block::GOLD_BLOCK).await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&dest, &Block::GOLD_BLOCK)
                .await?;
            helper.set_block_default(&power, &Block::AIR).await
        })
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&dest, &Block::GOLD_BLOCK)
                .await?;
            helper.assert_block_present(&cargo, &Block::AIR).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_pushes_downward_and_retracts(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let piston = BlockPos::new(1, 4, 1);
    let cargo = BlockPos::new(1, 3, 1);
    let dest = BlockPos::new(1, 2, 1);
    let power = BlockPos::new(2, 4, 1);

    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::Down))
        .await?;
    helper.set_block_default(&cargo, &Block::GOLD_BLOCK).await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper.assert_block_present(&dest, &Block::GOLD_BLOCK).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_pushes_north_and_south(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 6).await?;
    let p_north = BlockPos::new(1, 1, 3);
    let p_south = BlockPos::new(3, 1, 2);

    helper
        .set_block(&p_north, piston_state(&Block::PISTON, Facing::North))
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::IRON_BLOCK)
        .await?;
    helper
        .set_block(&p_south, piston_state(&Block::PISTON, Facing::South))
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 3), &Block::IRON_BLOCK)
        .await?;

    helper
        .set_block_default(&BlockPos::new(1, 2, 3), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 2, 2), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&BlockPos::new(1, 1, 1), &Block::IRON_BLOCK)
                .await?;
            helper
                .assert_block_present(&BlockPos::new(3, 1, 4), &Block::IRON_BLOCK)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_pushes_west_and_east(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    let p_west = BlockPos::new(3, 1, 1);
    let p_east = BlockPos::new(2, 1, 3);

    helper
        .set_block(&p_west, piston_state(&Block::PISTON, Facing::West))
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::DIAMOND_BLOCK)
        .await?;
    helper
        .set_block(&p_east, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 3), &Block::DIAMOND_BLOCK)
        .await?;

    helper
        .set_block_default(&BlockPos::new(3, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 2, 3), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&BlockPos::new(1, 1, 1), &Block::DIAMOND_BLOCK)
                .await?;
            helper
                .assert_block_present(&BlockPos::new(4, 1, 3), &Block::DIAMOND_BLOCK)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sticky_piston_pulls_upward_and_downward(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let piston = BlockPos::new(1, 1, 1);
    let cargo = BlockPos::new(1, 2, 1);
    let pushed = BlockPos::new(1, 3, 1);
    let power = BlockPos::new(2, 1, 1);

    helper
        .set_block(&piston, piston_state(&Block::STICKY_PISTON, Facing::Up))
        .await?;
    helper
        .set_block_default(&cargo, &Block::EMERALD_BLOCK)
        .await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&pushed, &Block::EMERALD_BLOCK)
                .await?;
            helper.set_block_default(&power, &Block::AIR).await
        })
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&cargo, &Block::EMERALD_BLOCK)
                .await?;
            helper.assert_block_present(&pushed, &Block::AIR).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_pushes_multi_block_row(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 3).await?;
    let piston = BlockPos::new(1, 1, 1);
    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::East))
        .await?;
    for x in 2..=5 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::COPPER_BLOCK)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            for x in 3..=6 {
                helper
                    .assert_block_present(&BlockPos::new(x, 1, 1), &Block::COPPER_BLOCK)
                    .await?;
            }
            Ok(())
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_blocked_by_obsidian(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 3).await?;
    let piston = BlockPos::new(1, 1, 1);
    let obsidian = BlockPos::new(2, 1, 1);
    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&obsidian, &Block::OBSIDIAN)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            let ext =
                StickyPistonLikeProperties::from_state_id(helper.block_state_id(&piston).await)
                    .extended;
            helper.assert_true(!ext, "Piston must not extend into obsidian")?;
            helper
                .assert_block_present(&obsidian, &Block::OBSIDIAN)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_blocked_by_bedrock(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 3).await?;
    let piston = BlockPos::new(1, 1, 1);
    let bedrock = BlockPos::new(2, 1, 1);
    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper.set_block_default(&bedrock, &Block::BEDROCK).await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            let ext =
                StickyPistonLikeProperties::from_state_id(helper.block_state_id(&piston).await)
                    .extended;
            helper.assert_true(!ext, "Piston must not extend into bedrock")?;
            helper.assert_block_present(&bedrock, &Block::BEDROCK).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_destroys_fragile_torch_and_moss(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let p1 = BlockPos::new(1, 1, 1);
    let torch = BlockPos::new(2, 1, 1);
    let p2 = BlockPos::new(1, 1, 2);
    let carpet = BlockPos::new(2, 1, 2);

    helper
        .set_block(&p1, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper.set_block_default(&torch, &Block::TORCH).await?;
    helper
        .set_block(&p2, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&carpet, &Block::MOSS_CARPET)
        .await?;

    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 2), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&torch, &Block::PISTON_HEAD)
                .await?;
            helper
                .assert_block_present(&carpet, &Block::PISTON_HEAD)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sticky_piston_leaves_obsidian_unmoved(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 3).await?;
    let piston = BlockPos::new(1, 1, 1);
    let obsidian = BlockPos::new(3, 1, 1);
    let power = BlockPos::new(1, 2, 1);

    helper
        .set_block(&piston, piston_state(&Block::STICKY_PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&obsidian, &Block::OBSIDIAN)
        .await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(
            move |helper| async move { helper.set_block_default(&power, &Block::AIR).await },
        )
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&obsidian, &Block::OBSIDIAN)
                .await?;
            helper
                .assert_block_present(&BlockPos::new(2, 1, 1), &Block::AIR)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_piston_pushes_redstone_block_powers_lamp(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;
    let piston = BlockPos::new(1, 1, 1);
    let rs_block = BlockPos::new(2, 1, 1);
    let lamp = BlockPos::new(4, 1, 1);

    helper
        .set_block(&piston, piston_state(&Block::PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&rs_block, &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&lamp, &Block::REDSTONE_LAMP)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            let lit =
                RedstoneOreLikeProperties::from_state_id(helper.block_state_id(&lamp).await).lit;
            helper.assert_true(
                lit,
                "Expected piston-pushed redstone_block to light adjacent redstone_lamp",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sticky_piston_pulls_redstone_block_cuts_power(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;
    let piston = BlockPos::new(1, 1, 1);
    let rs_start = BlockPos::new(2, 1, 1);
    let lamp = BlockPos::new(4, 1, 1);
    let power = BlockPos::new(1, 2, 1);

    helper
        .set_block(&piston, piston_state(&Block::STICKY_PISTON, Facing::East))
        .await?;
    helper
        .set_block_default(&rs_start, &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&lamp, &Block::REDSTONE_LAMP)
        .await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(
            move |helper| async move { helper.set_block_default(&power, &Block::AIR).await },
        )
        .then_idle(8)
        .then_execute(move |helper| async move {
            let lit =
                RedstoneOreLikeProperties::from_state_id(helper.block_state_id(&lamp).await).lit;
            helper.assert_true(
                !lit,
                "Expected lamp to turn off after sticky piston retracted redstone_block",
            )
        })
        .then_succeed();
    Ok(())
}
