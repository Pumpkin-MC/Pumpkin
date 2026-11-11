use pumpkin_data::{
    Block,
    block_properties::{
        HorizontalFacing, LanternLikeProperties, OakFenceGateLikeProperties,
        WallTorchLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_cactus_breaks_when_solid_block_placed_adjacent(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let sand = BlockPos::new(2, 1, 2);
    let cactus = BlockPos::new(2, 2, 2);
    test.set_block_default(&sand, &Block::SAND).await?;
    test.set_block_default(&cactus, &Block::CACTUS).await?;
    test.set_block_default(&BlockPos::new(3, 2, 2), &Block::STONE)
        .await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&cactus, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_cactus_breaks_when_sand_support_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let sand = BlockPos::new(2, 1, 2);
    let cactus = BlockPos::new(2, 2, 2);
    test.set_block_default(&sand, &Block::SAND).await?;
    test.set_block_default(&cactus, &Block::CACTUS).await?;
    test.set_block_default(&sand, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&cactus, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sugar_cane_breaks_when_water_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let sand = BlockPos::new(2, 1, 2);
    let water = BlockPos::new(3, 1, 2);
    let cane = BlockPos::new(2, 2, 2);

    for wall in [
        BlockPos::new(4, 1, 2),
        BlockPos::new(3, 1, 1),
        BlockPos::new(3, 1, 3),
    ] {
        test.set_block_default(&wall, &Block::STONE).await?;
    }
    test.set_block_default(&sand, &Block::SAND).await?;
    test.set_block_default(&water, &Block::WATER).await?;
    test.set_block_default(&cane, &Block::SUGAR_CANE).await?;

    // Remove water and trigger neighbor update on sugar cane
    test.set_block_default(&water, &Block::STONE).await?;
    test.set_block_default(&BlockPos::new(1, 2, 2), &Block::STONE)
        .await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&cane, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_sugar_cane_column_cascades_break(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let sand = BlockPos::new(2, 1, 2);
    let c1 = BlockPos::new(2, 2, 2);
    let c2 = BlockPos::new(2, 3, 2);
    test.set_block_default(&sand, &Block::SAND).await?;
    test.set_block_default(&c1, &Block::SUGAR_CANE).await?;
    test.set_block_default(&c2, &Block::SUGAR_CANE).await?;

    test.set_block_default(&sand, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(4)
        .then_execute(move |test| async move {
            test.assert_block_present(&c1, &Block::AIR).await?;
            test.assert_block_present(&c2, &Block::AIR).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_torch_and_soul_torch_break_without_support(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let s1 = BlockPos::new(1, 1, 1);
    let t1 = BlockPos::new(1, 2, 1);
    let s2 = BlockPos::new(3, 1, 1);
    let t2 = BlockPos::new(3, 2, 1);

    test.set_block_default(&s1, &Block::STONE).await?;
    test.set_block_default(&t1, &Block::TORCH).await?;
    test.set_block_default(&s2, &Block::STONE).await?;
    test.set_block_default(&t2, &Block::SOUL_TORCH).await?;

    test.set_block_default(&s1, &Block::AIR).await?;
    test.set_block_default(&s2, &Block::AIR).await?;

    test.assert_block_present(&t1, &Block::AIR).await?;
    test.assert_block_present(&t2, &Block::AIR).await?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_wall_torch_breaks_when_wall_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let wall = BlockPos::new(2, 1, 2);
    let torch = BlockPos::new(2, 1, 1);
    test.set_block_default(&wall, &Block::STONE).await?;

    let mut wt = WallTorchLikeProperties::default(&Block::WALL_TORCH);
    wt.facing = HorizontalFacing::North;
    test.set_block(&torch, wt.to_state_id(&Block::WALL_TORCH))
        .await?;

    test.set_block_default(&wall, &Block::AIR).await?;
    test.assert_block_present(&torch, &Block::AIR).await?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_lantern_standing_breaks_when_floor_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let support = BlockPos::new(1, 1, 1);
    let lantern = BlockPos::new(1, 2, 1);
    test.set_block_default(&support, &Block::STONE).await?;
    test.set_block_default(&lantern, &Block::LANTERN).await?;
    test.set_block_default(&support, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&lantern, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_lantern_hanging_breaks_when_ceiling_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let ceiling = BlockPos::new(1, 3, 1);
    let lantern = BlockPos::new(1, 2, 1);
    test.set_block_default(&ceiling, &Block::STONE).await?;
    let mut props = LanternLikeProperties::default(&Block::SOUL_LANTERN);
    props.hanging = true;
    test.set_block(&lantern, props.to_state_id(&Block::SOUL_LANTERN))
        .await?;

    test.set_block_default(&ceiling, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&lantern, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_lantern_on_fence_gate_breaks_when_gate_opens(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let gate = BlockPos::new(1, 1, 1);
    let lantern = BlockPos::new(1, 2, 1);
    let mut g_props = OakFenceGateLikeProperties::default(&Block::OAK_FENCE_GATE);
    g_props.open = false;
    test.set_block(&gate, g_props.to_state_id(&Block::OAK_FENCE_GATE))
        .await?;
    test.set_block_default(&lantern, &Block::LANTERN).await?;

    // Power the fence gate so it opens -> lantern above loses support and breaks on scheduled tick
    test.set_block_default(&BlockPos::new(2, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(
            move |test| async move { test.assert_block_present(&lantern, &Block::AIR).await },
        )
        .then_succeed();
    Ok(())
}

pub(super) async fn test_cake_and_candle_cake_break_when_floor_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let s1 = BlockPos::new(1, 1, 1);
    let cake = BlockPos::new(1, 2, 1);
    let s2 = BlockPos::new(3, 1, 1);
    let candle_cake = BlockPos::new(3, 2, 1);

    test.set_block_default(&s1, &Block::STONE).await?;
    test.set_block_default(&cake, &Block::CAKE).await?;
    test.set_block_default(&s2, &Block::STONE).await?;
    test.set_block_default(&candle_cake, &Block::CANDLE_CAKE)
        .await?;

    test.set_block_default(&s1, &Block::AIR).await?;
    test.set_block_default(&s2, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| async move {
            test.assert_block_present(&cake, &Block::AIR).await?;
            test.assert_block_present(&candle_cake, &Block::AIR).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_carpet_and_moss_carpet_break_when_floor_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let s1 = BlockPos::new(1, 1, 1);
    let c1 = BlockPos::new(1, 2, 1);
    let s2 = BlockPos::new(3, 1, 1);
    let c2 = BlockPos::new(3, 2, 1);

    test.set_block_default(&s1, &Block::STONE).await?;
    test.set_block_default(&c1, &Block::WHITE_CARPET).await?;
    test.set_block_default(&s2, &Block::STONE).await?;
    test.set_block_default(&c2, &Block::MOSS_CARPET).await?;

    test.set_block_default(&s1, &Block::AIR).await?;
    test.set_block_default(&s2, &Block::AIR).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| async move {
            test.assert_block_present(&c1, &Block::AIR).await?;
            test.assert_block_present(&c2, &Block::AIR).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_pressure_plate_breaks_when_floor_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let s1 = BlockPos::new(1, 1, 1);
    let p1 = BlockPos::new(1, 2, 1);
    let s2 = BlockPos::new(3, 1, 1);
    let p2 = BlockPos::new(3, 2, 1);

    test.set_block_default(&s1, &Block::STONE).await?;
    test.set_block_default(&p1, &Block::STONE_PRESSURE_PLATE)
        .await?;
    test.set_block_default(&s2, &Block::STONE).await?;
    test.set_block_default(&p2, &Block::LIGHT_WEIGHTED_PRESSURE_PLATE)
        .await?;

    test.set_block_default(&s1, &Block::AIR).await?;
    test.set_block_default(&s2, &Block::AIR).await?;

    test.assert_block_present(&p1, &Block::AIR).await?;
    test.assert_block_present(&p2, &Block::AIR).await?;
    test.succeed();
    Ok(())
}
