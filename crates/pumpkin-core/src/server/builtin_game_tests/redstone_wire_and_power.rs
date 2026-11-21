use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        EastRedstone, HorizontalFacing, NorthRedstone, RedstoneWireLikeProperties,
        RepeaterLikeProperties, SouthRedstone, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

fn ew_wire_state() -> BlockStateId {
    let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    props.east = EastRedstone::Side;
    props.west = WestRedstone::Side;
    props.north = NorthRedstone::None;
    props.south = SouthRedstone::None;
    props.to_state_id(&Block::REDSTONE_WIRE)
}

fn ns_wire_state() -> BlockStateId {
    let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    props.north = NorthRedstone::Side;
    props.south = SouthRedstone::Side;
    props.east = EastRedstone::None;
    props.west = WestRedstone::None;
    props.to_state_id(&Block::REDSTONE_WIRE)
}

async fn wire_power(helper: &GameTestHelper, pos: &BlockPos) -> u8 {
    RedstoneWireLikeProperties::from_state_id(helper.block_state_id(pos).await).power
}

pub(super) async fn test_redstone_wire_attenuation_1_to_4(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 7, 3).await?;
    let wire = ew_wire_state();
    for x in 2..=5 {
        helper.set_block(&BlockPos::new(x, 1, 1), wire).await?;
    }
    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    for (x, expected) in [(2, 15), (3, 14), (4, 13), (5, 12)] {
        let actual = wire_power(&helper, &BlockPos::new(x, 1, 1)).await;
        helper.assert_true(
            actual == expected,
            format!("Expected wire at x={x} to have power {expected}, got {actual}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_attenuation_5_to_8(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 10, 3).await?;
    let wire = ew_wire_state();
    for x in 1..=8 {
        helper.set_block(&BlockPos::new(x, 1, 1), wire).await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    for (x, expected) in [(5, 11), (6, 10), (7, 9), (8, 8)] {
        let actual = wire_power(&helper, &BlockPos::new(x, 1, 1)).await;
        helper.assert_true(
            actual == expected,
            format!("Expected wire at x={x} to have power {expected}, got {actual}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_attenuation_9_to_12(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 8).await?;
    // Snake path inside 8x8: (1,1)->(6,1) [6 blocks: 15..10], (6,2)->(6,7) [6 blocks: 9..4]
    for x in 1..=6 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_WIRE)
            .await?;
    }
    for z in 2..=7 {
        helper
            .set_block_default(&BlockPos::new(6, 1, z), &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    for (z, expected) in [(4, 7), (5, 6), (6, 5), (7, 4)] {
        let actual = wire_power(&helper, &BlockPos::new(6, 1, z)).await;
        helper.assert_true(
            actual == expected,
            format!("Expected wire at (6,1,{z}) to have power {expected}, got {actual}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_attenuation_13_to_15(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 8).await?;
    // Snake path of 16 wires: (1..=6, 1) [6: 15..10], (6, 2..=6) [5: 9..5], (5..=1, 6) [5: 4..0]
    for x in 1..=6 {
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_WIRE)
            .await?;
    }
    for z in 2..=6 {
        helper
            .set_block_default(&BlockPos::new(6, 1, z), &Block::REDSTONE_WIRE)
            .await?;
    }
    for x in (1..=5).rev() {
        helper
            .set_block_default(&BlockPos::new(x, 1, 6), &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    for (x, expected) in [(4, 3), (3, 2), (2, 1), (1, 0)] {
        let actual = wire_power(&helper, &BlockPos::new(x, 1, 6)).await;
        helper.assert_true(
            actual == expected,
            format!("Expected wire at ({x},1,6) to have power {expected}, got {actual}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_corner_north_east(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(1, 1, 3),
        BlockPos::new(1, 1, 2),
        BlockPos::new(1, 1, 1),
        BlockPos::new(2, 1, 1),
        BlockPos::new(3, 1, 1),
    ] {
        helper
            .set_block_default(&pos, &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(1, 1, 4), &Block::REDSTONE_BLOCK)
        .await?;

    let corner = wire_power(&helper, &BlockPos::new(1, 1, 1)).await;
    let end = wire_power(&helper, &BlockPos::new(3, 1, 1)).await;
    helper.assert_true(
        corner == 13 && end == 11,
        format!("Expected corner=13 and end=11, got {corner} and {end}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_corner_south_west(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(3, 1, 1),
        BlockPos::new(3, 1, 2),
        BlockPos::new(3, 1, 3),
        BlockPos::new(2, 1, 3),
        BlockPos::new(1, 1, 3),
    ] {
        helper
            .set_block_default(&pos, &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(3, 1, 0), &Block::REDSTONE_BLOCK)
        .await?;

    let corner = wire_power(&helper, &BlockPos::new(3, 1, 3)).await;
    let end = wire_power(&helper, &BlockPos::new(1, 1, 3)).await;
    helper.assert_true(
        corner == 13 && end == 11,
        format!("Expected corner=13 and end=11, got {corner} and {end}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_t_junction_split(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(2, 1, 1),
        BlockPos::new(2, 1, 2),
        BlockPos::new(1, 1, 2),
        BlockPos::new(3, 1, 2),
    ] {
        helper
            .set_block_default(&pos, &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(2, 1, 0), &Block::REDSTONE_BLOCK)
        .await?;

    let center = wire_power(&helper, &BlockPos::new(2, 1, 2)).await;
    let west = wire_power(&helper, &BlockPos::new(1, 1, 2)).await;
    let east = wire_power(&helper, &BlockPos::new(3, 1, 2)).await;
    helper.assert_true(
        center == 14 && west == 13 && east == 13,
        format!("Expected T-junction center=14, west=13, east=13; got {center}, {west}, {east}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_cross_junction_four_way(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for pos in [
        BlockPos::new(2, 1, 1),
        BlockPos::new(2, 1, 2),
        BlockPos::new(2, 1, 3),
        BlockPos::new(1, 1, 2),
        BlockPos::new(3, 1, 2),
    ] {
        helper
            .set_block_default(&pos, &Block::REDSTONE_WIRE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(2, 1, 0), &Block::REDSTONE_BLOCK)
        .await?;

    let south = wire_power(&helper, &BlockPos::new(2, 1, 3)).await;
    let west = wire_power(&helper, &BlockPos::new(1, 1, 2)).await;
    let east = wire_power(&helper, &BlockPos::new(3, 1, 2)).await;
    helper.assert_true(
        south == 13 && west == 13 && east == 13,
        format!(
            "Expected all 3 cross arms to have power 13, got south={south}, west={west}, east={east}"
        ),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_step_up_climb(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 1), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(4, 1, 1), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 2, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(4, 2, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    let p_low = wire_power(&helper, &BlockPos::new(2, 1, 1)).await;
    let p_step = wire_power(&helper, &BlockPos::new(3, 2, 1)).await;
    let p_top = wire_power(&helper, &BlockPos::new(4, 2, 1)).await;
    helper.assert_true(
        p_low == 15 && p_step == 14 && p_top == 13,
        format!("Expected step-up powers 15->14->13, got {p_low}->{p_step}->{p_top}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_step_down_drop(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 2, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(4, 1, 1), &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    let p_high = wire_power(&helper, &BlockPos::new(2, 2, 1)).await;
    let p_drop = wire_power(&helper, &BlockPos::new(3, 1, 1)).await;
    let p_low = wire_power(&helper, &BlockPos::new(4, 1, 1)).await;
    helper.assert_true(
        p_high == 15 && p_drop == 14 && p_low == 13,
        format!("Expected step-down powers 15->14->13, got {p_high}->{p_drop}->{p_low}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_dual_sources_max_power(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 7, 3).await?;
    let wire = ew_wire_state();
    for x in 1..=5 {
        helper.set_block(&BlockPos::new(x, 1, 1), wire).await?;
    }
    helper
        .set_block_default(&BlockPos::new(0, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(6, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    // Middle wire at x=3 receives 13 from x=0 and 13 from x=6; x=4 receives 14 from x=6
    let p3 = wire_power(&helper, &BlockPos::new(3, 1, 1)).await;
    let p4 = wire_power(&helper, &BlockPos::new(4, 1, 1)).await;
    helper.assert_true(
        p3 == 13 && p4 == 14,
        format!("Expected p3=13 and p4=14 between dual sources, got {p3} and {p4}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_source_removal_deenergizes(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 3).await?;
    let wire = ew_wire_state();
    for x in 2..=4 {
        helper.set_block(&BlockPos::new(x, 1, 1), wire).await?;
    }
    let src = BlockPos::new(1, 1, 1);
    helper
        .set_block_default(&src, &Block::REDSTONE_BLOCK)
        .await?;
    helper.assert_true(
        wire_power(&helper, &BlockPos::new(4, 1, 1)).await == 13,
        "Expected wire to be powered before source removal",
    )?;

    helper.set_block_default(&src, &Block::AIR).await?;
    for x in 2..=4 {
        let p = wire_power(&helper, &BlockPos::new(x, 1, 1)).await;
        helper.assert_true(
            p == 0,
            format!("Expected wire at x={x} to de-energize to 0, got {p}"),
        )?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_repeater_reboosts_signal(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 7).await?;
    let wire = ns_wire_state();
    for z in 1..=3 {
        helper.set_block(&BlockPos::new(1, 1, z), wire).await?;
    }
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = 1;
    helper
        .set_block(&BlockPos::new(1, 1, 4), rep.to_state_id(&Block::REPEATER))
        .await?;
    helper.set_block(&BlockPos::new(1, 1, 5), wire).await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 0), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            let before_rep = wire_power(&helper, &BlockPos::new(1, 1, 3)).await;
            let after_rep = wire_power(&helper, &BlockPos::new(1, 1, 5)).await;
            helper.assert_true(
                before_rep == 13 && after_rep == 15,
                format!("Expected wire before repeater=13 and after repeater=15, got {before_rep} and {after_rep}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_wire_strong_power_through_stone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 6).await?;
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = 1;
    helper
        .set_block(&BlockPos::new(1, 1, 2), rep.to_state_id(&Block::REPEATER))
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 3), &Block::STONE)
        .await?;
    helper
        .set_block(&BlockPos::new(1, 1, 4), ns_wire_state())
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            let out = wire_power(&helper, &BlockPos::new(1, 1, 4)).await;
            helper.assert_true(
                out == 15,
                format!("Expected strong-powered stone block to energize wire to 15, got {out}"),
            )
        })
        .then_succeed();
    Ok(())
}
