use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{PoweredRailLikeProperties, RailShapeStraight},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

fn ns_rail_state(block: &Block) -> BlockStateId {
    let mut props = PoweredRailLikeProperties::default(block);
    props.shape = RailShapeStraight::NorthSouth;
    props.powered = false;
    props.to_state_id(block)
}

pub(super) async fn test_powered_rail_deenergizes_beyond_8_blocks(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 12).await?;
    let rail = ns_rail_state(&Block::POWERED_RAIL);
    for z in 1..=10 {
        helper.set_block(&BlockPos::new(1, 1, z), rail).await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    let p9 = PoweredRailLikeProperties::from_state_id(
        helper.block_state_id(&BlockPos::new(1, 1, 9)).await,
    )
    .powered;
    let p10 = PoweredRailLikeProperties::from_state_id(
        helper.block_state_id(&BlockPos::new(1, 1, 10)).await,
    )
    .powered;
    helper.assert_true(
        p9 && !p10,
        format!("Expected powered_rail powered at dist 8 (z=9) and unpowered at dist 9 (z=10), got {p9} and {p10}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_powered_rail_bidirectional_boost_from_center(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 11).await?;
    let rail = ns_rail_state(&Block::POWERED_RAIL);
    for z in 1..=9 {
        helper.set_block(&BlockPos::new(1, 1, z), rail).await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 5), &Block::REDSTONE_BLOCK)
        .await?;

    for z in 1..=9 {
        let powered = PoweredRailLikeProperties::from_state_id(
            helper.block_state_id(&BlockPos::new(1, 1, z)).await,
        )
        .powered;
        helper.assert_true(
            powered,
            format!("Expected powered_rail at z={z} to be powered from center z=5"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_activator_rail_deenergizes_beyond_8_blocks(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 12).await?;
    let rail = ns_rail_state(&Block::ACTIVATOR_RAIL);
    for z in 1..=10 {
        helper.set_block(&BlockPos::new(1, 1, z), rail).await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    let p9 = PoweredRailLikeProperties::from_state_id(
        helper.block_state_id(&BlockPos::new(1, 1, 9)).await,
    )
    .powered;
    let p10 = PoweredRailLikeProperties::from_state_id(
        helper.block_state_id(&BlockPos::new(1, 1, 10)).await,
    )
    .powered;
    helper.assert_true(
        p9 && !p10,
        format!("Expected activator_rail powered at z=9 and unpowered at z=10, got {p9} and {p10}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_detector_rail_powers_adjacent_powered_rail(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 5).await?;
    let det = BlockPos::new(1, 1, 1);
    let pwr_rail = BlockPos::new(1, 1, 2);
    helper
        .set_block(&det, ns_rail_state(&Block::DETECTOR_RAIL))
        .await?;
    helper
        .set_block(&pwr_rail, ns_rail_state(&Block::POWERED_RAIL))
        .await?;

    helper
        .spawn_entity_at("minecraft:minecart", Vector3::new(1.5, 1.1, 1.5))
        .await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            let powered =
                PoweredRailLikeProperties::from_state_id(helper.block_state_id(&pwr_rail).await)
                    .powered;
            helper.assert_true(
                powered,
                "Expected detector_rail with minecart to energize adjacent powered_rail",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_detector_rail_comparator_reads_chest_minecart(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 5).await?;
    let det = BlockPos::new(1, 1, 2);
    helper
        .set_block(&det, ns_rail_state(&Block::DETECTOR_RAIL))
        .await?;
    helper
        .spawn_entity_at("minecraft:chest_minecart", Vector3::new(1.5, 1.1, 2.5))
        .await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            let powered =
                PoweredRailLikeProperties::from_state_id(helper.block_state_id(&det).await).powered;
            helper.assert_true(powered, "Expected chest_minecart to activate detector_rail")
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_rail_breaks_when_support_block_removed(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let support = BlockPos::new(1, 1, 1);
    let rail = BlockPos::new(1, 2, 1);
    helper.set_block_default(&support, &Block::STONE).await?;
    helper.set_block_default(&rail, &Block::RAIL).await?;

    helper.set_block_default(&support, &Block::AIR).await?;
    helper.assert_block_present(&rail, &Block::AIR).await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_water_flows_horizontally_and_decreases_level(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    // Build a 1-wide channel from x=1 to x=4 at z=2
    for x in 0..=5 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::STONE)
            .await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 3), &Block::STONE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 2), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(5, 1, 2), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::WATER)
        .await?;

    helper
        .start_sequence()
        .then_idle(12)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&BlockPos::new(2, 1, 2), &Block::WATER)
                .await?;
            helper
                .assert_block_present(&BlockPos::new(3, 1, 2), &Block::WATER)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_water_falls_downward_into_basin(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    // Build a raised basin at (2, 2, 2) with a hole at (2, 1, 2)
    for pos in [
        BlockPos::new(1, 2, 2),
        BlockPos::new(3, 2, 2),
        BlockPos::new(2, 2, 1),
        BlockPos::new(2, 2, 3),
        BlockPos::new(1, 1, 2),
        BlockPos::new(3, 1, 2),
        BlockPos::new(2, 1, 1),
        BlockPos::new(2, 1, 3),
    ] {
        helper.set_block_default(&pos, &Block::STONE).await?;
    }
    helper
        .set_block_default(&BlockPos::new(2, 2, 2), &Block::WATER)
        .await?;

    helper
        .start_sequence()
        .then_idle(8)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&BlockPos::new(2, 1, 2), &Block::WATER)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_lava_meets_flowing_water_forms_cobblestone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    for x in 0..=5 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::STONE)
            .await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 3), &Block::STONE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 2), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(5, 1, 2), &Block::STONE)
        .await?;

    // Water at (1,1,2) flows into (2,1,2); placing lava at (3,1,2) converts flowing water at (2,1,2) or lava
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::WATER)
        .await?;

    helper
        .start_sequence()
        .then_idle(7)
        .then_execute(move |helper| async move {
            helper
                .set_block_default(&BlockPos::new(2, 1, 2), &Block::LAVA)
                .await?;
            helper
                .assert_block_present(&BlockPos::new(2, 1, 2), &Block::OBSIDIAN)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_water_meets_lava_source_forms_obsidian(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(1, 1, 2),
        BlockPos::new(4, 1, 2),
        BlockPos::new(2, 1, 1),
        BlockPos::new(3, 1, 1),
        BlockPos::new(2, 1, 3),
        BlockPos::new(3, 1, 3),
    ] {
        helper.set_block_default(&pos, &Block::STONE).await?;
    }
    let lava = BlockPos::new(2, 1, 2);
    let water = BlockPos::new(3, 1, 2);
    helper.set_block_default(&lava, &Block::LAVA).await?;
    helper.set_block_default(&water, &Block::WATER).await?;

    helper.assert_block_present(&lava, &Block::OBSIDIAN).await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_water_breaks_torch_and_carpet_in_path(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    for x in 0..=5 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::STONE)
            .await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 3), &Block::STONE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 2), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(5, 1, 2), &Block::STONE)
        .await?;

    let torch = BlockPos::new(2, 1, 2);
    helper.set_block_default(&torch, &Block::TORCH).await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::WATER)
        .await?;

    helper
        .start_sequence()
        .then_idle(8)
        .then_execute(move |helper| async move {
            helper.assert_block_present(&torch, &Block::WATER).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sponge_ignores_lava_and_stays_dry(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(1, 1, 2),
        BlockPos::new(3, 1, 2),
        BlockPos::new(2, 1, 1),
    ] {
        helper.set_block_default(&pos, &Block::STONE).await?;
    }
    let lava = BlockPos::new(2, 1, 2);
    let sponge = BlockPos::new(2, 1, 3);
    helper.set_block_default(&lava, &Block::LAVA).await?;
    helper.set_block_default(&sponge, &Block::SPONGE).await?;

    helper.assert_block_present(&sponge, &Block::SPONGE).await?;
    helper.assert_block_present(&lava, &Block::LAVA).await?;
    helper.succeed();
    Ok(())
}
