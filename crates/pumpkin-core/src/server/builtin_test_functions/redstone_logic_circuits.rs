use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        AttachFace, EastRedstone, FurnaceLikeProperties, HorizontalFacing, LeverLikeProperties,
        NorthRedstone, RedstoneOreLikeProperties, RedstoneWireLikeProperties,
        RepeaterLikeProperties, SouthRedstone, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

fn ns_wire() -> BlockStateId {
    let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    props.north = NorthRedstone::Side;
    props.south = SouthRedstone::Side;
    props.east = EastRedstone::None;
    props.west = WestRedstone::None;
    props.to_state_id(&Block::REDSTONE_WIRE)
}

pub(super) async fn test_redstone_double_inverter_buffer(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let s1 = BlockPos::new(2, 1, 2);
    let t1 = BlockPos::new(2, 2, 2);
    let s2 = BlockPos::new(2, 3, 2);
    let t2 = BlockPos::new(2, 4, 2);
    let lever_pos = BlockPos::new(2, 1, 1);

    test.set_block_default(&s1, &Block::STONE).await?;
    test.set_block_default(&t1, &Block::REDSTONE_TORCH).await?;
    test.set_block_default(&s2, &Block::STONE).await?;
    test.set_block_default(&t2, &Block::REDSTONE_TORCH).await?;

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Wall;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    test.set_block(&lever_pos, lever_props.to_state_id(&Block::LEVER))
        .await?;

    // Pull lever on s1 -> t1 turns off -> s2 loses strong power -> t2 stays/turns lit
    test.pull_lever(&lever_pos).await?;

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            let t1_lit =
                RedstoneOreLikeProperties::from_state_id(test.block_state_id(&t1).await).lit;
            let t2_lit =
                RedstoneOreLikeProperties::from_state_id(test.block_state_id(&t2).await).lit;
            test.assert_true(
                !t1_lit && t2_lit,
                format!(
                    "Expected t1=unlit and double-inverted t2=lit, got t1={t1_lit}, t2={t2_lit}"
                ),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_wall_torch_inverter(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let support = BlockPos::new(1, 1, 1);
    let torch = BlockPos::new(1, 1, 2);
    let lamp = BlockPos::new(1, 1, 3);

    test.set_block_default(&support, &Block::STONE).await?;
    let mut wt = FurnaceLikeProperties::default(&Block::REDSTONE_WALL_TORCH);
    wt.facing = HorizontalFacing::South;
    test.set_block(&torch, wt.to_state_id(&Block::REDSTONE_WALL_TORCH))
        .await?;
    test.set_block_default(&lamp, &Block::REDSTONE_LAMP).await?;

    // Lamp is lit by the wall torch initially; powering support turns torch off -> lamp turns off
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(8)
        .then_execute(move |test| async move {
            let lit =
                RedstoneOreLikeProperties::from_state_id(test.block_state_id(&lamp).await).lit;
            test.assert_true(
                !lit,
                "Expected lamp to turn off after wall torch inverter was powered",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_or_gate_two_inputs(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    for x in 1..=3 {
        test.set_block_default(&BlockPos::new(x, 1, 2), &Block::REDSTONE_WIRE)
            .await?;
    }
    let mut lever_off = LeverLikeProperties::default(&Block::LEVER);
    lever_off.face = AttachFace::Floor;
    lever_off.powered = false;
    let off_id = lever_off.to_state_id(&Block::LEVER);
    lever_off.powered = true;
    let on_id = lever_off.to_state_id(&Block::LEVER);

    let l1 = BlockPos::new(1, 1, 1);
    let l2 = BlockPos::new(3, 1, 1);
    let out = BlockPos::new(2, 1, 2);
    test.set_block(&l1, off_id).await?;
    test.set_block(&l2, off_id).await?;
    test.assert_true(
        RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out).await).power == 0,
        "OR gate should be 0 when both levers are off",
    )?;

    test.set_block(&l1, on_id).await?;
    test.assert_true(
        RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out).await).power > 0,
        "OR gate should be >0 when lever 1 is on",
    )?;

    test.set_block(&l1, off_id).await?;
    test.set_block(&l2, on_id).await?;
    test.assert_true(
        RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out).await).power > 0,
        "OR gate should be >0 when lever 2 is on",
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_redstone_nor_gate_two_inputs(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let gate_block = BlockPos::new(2, 1, 2);
    let torch_pos = BlockPos::new(2, 1, 3);
    let out_wire = BlockPos::new(2, 1, 4);
    let lever_pos = BlockPos::new(2, 1, 1);

    test.set_block_default(&gate_block, &Block::STONE).await?;
    let mut wt = FurnaceLikeProperties::default(&Block::REDSTONE_WALL_TORCH);
    wt.facing = HorizontalFacing::South;
    test.set_block(&torch_pos, wt.to_state_id(&Block::REDSTONE_WALL_TORCH))
        .await?;
    test.set_block(&out_wire, ns_wire()).await?;

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Wall;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    test.set_block(&lever_pos, lever_props.to_state_id(&Block::LEVER))
        .await?;

    // Pulling lever strongly powers gate_block -> turns NOR output torch & wire off
    test.pull_lever(&lever_pos).await?;

    test.start_sequence()
        .then_idle(4)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await)
                .power;
            test.assert_true(
                p == 0,
                format!("Expected NOR output 0 when input 1 is high, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_and_gate_three_torches(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let b1 = BlockPos::new(1, 1, 2);
    let b2 = BlockPos::new(3, 1, 2);
    let t1 = BlockPos::new(1, 2, 2);
    let t2 = BlockPos::new(3, 2, 2);
    let mid_stone = BlockPos::new(2, 1, 2);
    let mid_wire = BlockPos::new(2, 2, 2);
    let out_torch = BlockPos::new(2, 1, 3);
    let l1 = BlockPos::new(1, 1, 1);
    let l2 = BlockPos::new(3, 1, 1);

    test.set_block_default(&b1, &Block::STONE).await?;
    test.set_block_default(&b2, &Block::STONE).await?;
    test.set_block_default(&mid_stone, &Block::STONE).await?;
    test.set_block_default(&t1, &Block::REDSTONE_TORCH).await?;
    test.set_block_default(&t2, &Block::REDSTONE_TORCH).await?;
    test.set_block_default(&mid_wire, &Block::REDSTONE_WIRE)
        .await?;

    let mut wt = FurnaceLikeProperties::default(&Block::REDSTONE_WALL_TORCH);
    wt.facing = HorizontalFacing::South;
    wt.lit = false;
    test.set_block(&out_torch, wt.to_state_id(&Block::REDSTONE_WALL_TORCH))
        .await?;

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Wall;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    let lever_state = lever_props.to_state_id(&Block::LEVER);
    test.set_block(&l1, lever_state).await?;
    test.set_block(&l2, lever_state).await?;

    // Pull both levers -> t1 & t2 turn off -> mid_wire becomes 0 -> out_torch turns on!
    test.pull_lever(&l1).await?;
    test.pull_lever(&l2).await?;

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            let lit =
                FurnaceLikeProperties::from_state_id(test.block_state_id(&out_torch).await).lit;
            test.assert_true(
                lit,
                "Expected AND gate output torch lit when both inputs are powered",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_nand_gate_two_torches(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let b1 = BlockPos::new(1, 1, 2);
    let b2 = BlockPos::new(3, 1, 2);
    let t1 = BlockPos::new(1, 2, 2);
    let t2 = BlockPos::new(3, 2, 2);
    let mid_stone = BlockPos::new(2, 1, 2);
    let mid_wire = BlockPos::new(2, 2, 2);
    let l1 = BlockPos::new(1, 1, 1);
    let l2 = BlockPos::new(3, 1, 1);

    test.set_block_default(&b1, &Block::STONE).await?;
    test.set_block_default(&b2, &Block::STONE).await?;
    test.set_block_default(&mid_stone, &Block::STONE).await?;
    test.set_block_default(&t1, &Block::REDSTONE_TORCH).await?;
    test.set_block_default(&t2, &Block::REDSTONE_TORCH).await?;
    test.set_block_default(&mid_wire, &Block::REDSTONE_WIRE)
        .await?;

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Wall;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    let lever_state = lever_props.to_state_id(&Block::LEVER);
    test.set_block(&l1, lever_state).await?;
    test.set_block(&l2, lever_state).await?;

    // Pull only l1 -> t2 stays lit -> NAND mid_wire stays powered
    test.pull_lever(&l1).await?;

    test.start_sequence()
        .then_idle(4)
        .then_execute(move |test| async move {
            let p_one =
                RedstoneWireLikeProperties::from_state_id(test.block_state_id(&mid_wire).await)
                    .power;
            test.assert_true(
                p_one > 0,
                "Expected NAND wire to remain powered when only 1 input is on",
            )?;
            test.pull_lever(&l2).await
        })
        .then_idle(4)
        .then_execute(move |test| async move {
            let p_both =
                RedstoneWireLikeProperties::from_state_id(test.block_state_id(&mid_wire).await)
                    .power;
            test.assert_true(
                p_both == 0,
                format!("Expected NAND wire to drop to 0 when both inputs are on, got {p_both}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_rs_nor_latch_memory(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    // Two inverters cross-coupled or a repeater self-lock latch
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = 1;
    let rep_pos = BlockPos::new(2, 1, 2);
    let lock_pos = BlockPos::new(1, 1, 2);
    let out_wire = BlockPos::new(2, 1, 3);
    let in_src = BlockPos::new(2, 1, 1);
    let lock_src = BlockPos::new(0, 1, 2);

    let mut lock_rep = RepeaterLikeProperties::default(&Block::REPEATER);
    lock_rep.facing = HorizontalFacing::West;
    lock_rep.delay = 1;

    test.set_block(&rep_pos, rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&lock_pos, lock_rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&out_wire, ns_wire()).await?;
    test.set_block_default(&in_src, &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(3)
        .then_execute(move |test| async move {
            // Lock the repeater while it is powered, then remove input power
            test.set_block_default(&lock_src, &Block::REDSTONE_BLOCK)
                .await
        })
        .then_idle(3)
        .then_execute(move |test| async move { test.set_block_default(&in_src, &Block::AIR).await })
        .then_idle(4)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await)
                .power;
            test.assert_true(
                p == 15,
                format!("Expected latched repeater to hold output 15 after input removed, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

async fn run_repeater_delay_test(
    test: GameTestContext,
    delay_setting: u8,
    early_ticks: u32,
    late_ticks: u32,
) -> GameTestResult<()> {
    build_stone_floor(&test, 3, 5).await?;
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = delay_setting;
    let rep_pos = BlockPos::new(1, 1, 2);
    let out_wire = BlockPos::new(1, 1, 3);
    test.set_block(&rep_pos, rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&out_wire, ns_wire()).await?;
    test.set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test
        .start_sequence()
        .then_idle(early_ticks)
        .then_execute(move |test| async move {
            let p =
                RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await)
                    .power;
            test.assert_true(
                p == 0,
                format!("Expected repeater (delay={delay_setting}) to still be 0 at tick {early_ticks}, got {p}"),
            )
        })
        .then_idle(late_ticks)
        .then_execute(move |test| async move {
            let p =
                RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await)
                    .power;
            test.assert_true(
                p == 15,
                format!("Expected repeater (delay={delay_setting}) to be 15 after delay, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_repeater_delay_2_ticks(
    test: GameTestContext,
) -> GameTestResult<()> {
    run_repeater_delay_test(test, 2, 2, 4).await
}

pub(super) async fn test_redstone_repeater_delay_3_ticks(
    test: GameTestContext,
) -> GameTestResult<()> {
    run_repeater_delay_test(test, 3, 4, 4).await
}

pub(super) async fn test_redstone_repeater_delay_4_ticks(
    test: GameTestContext,
) -> GameTestResult<()> {
    run_repeater_delay_test(test, 4, 6, 4).await
}

pub(super) async fn test_redstone_repeater_locking_holds_off(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = 1;
    let rep_pos = BlockPos::new(2, 1, 2);
    let out_wire = BlockPos::new(2, 1, 3);

    let mut lock_rep = RepeaterLikeProperties::default(&Block::REPEATER);
    lock_rep.facing = HorizontalFacing::West;
    lock_rep.delay = 1;
    let lock_pos = BlockPos::new(1, 1, 2);

    test.set_block(&rep_pos, rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&lock_pos, lock_rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&out_wire, ns_wire()).await?;
    // Power the side locking repeater first
    test.set_block_default(&BlockPos::new(0, 1, 2), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(3)
        .then_execute(move |test| async move {
            // Now apply rear input while locked
            test.set_block_default(&BlockPos::new(2, 1, 1), &Block::REDSTONE_BLOCK)
                .await
        })
        .then_idle(4)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await)
                .power;
            test.assert_true(
                p == 0,
                format!("Expected locked-off repeater output to stay 0, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_repeater_locking_holds_on(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let mut rep = RepeaterLikeProperties::default(&Block::REPEATER);
    rep.facing = HorizontalFacing::North;
    rep.delay = 1;
    rep.powered = true;
    rep.locked = true;
    let rep_pos = BlockPos::new(2, 1, 2);
    let out_wire = BlockPos::new(2, 1, 3);

    let mut lock_rep = RepeaterLikeProperties::default(&Block::REPEATER);
    lock_rep.facing = HorizontalFacing::West;
    lock_rep.delay = 1;
    lock_rep.powered = true;
    let lock_pos = BlockPos::new(1, 1, 2);

    test.set_block_default(&BlockPos::new(0, 1, 2), &Block::REDSTONE_BLOCK)
        .await?;
    test.set_block(&lock_pos, lock_rep.to_state_id(&Block::REPEATER))
        .await?;
    test.set_block(&out_wire, ns_wire()).await?;
    test.set_block(&rep_pos, rep.to_state_id(&Block::REPEATER))
        .await?;

    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&out_wire).await).power;
    test.assert_true(
        p == 15,
        format!("Expected locked-on repeater to power output wire to 15, got {p}"),
    )?;
    test.succeed();
    Ok(())
}
