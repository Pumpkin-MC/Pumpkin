use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        AttachFace, EastRedstone, HorizontalFacing, LeverLikeProperties,
        LightWeightedPressurePlateLikeProperties, NorthRedstone, RedstoneWireLikeProperties,
        SouthRedstone, StonePressurePlateLikeProperties, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

fn ns_wire() -> BlockStateId {
    let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    props.north = NorthRedstone::Side;
    props.south = SouthRedstone::Side;
    props.east = EastRedstone::None;
    props.west = WestRedstone::None;
    props.to_state_id(&Block::REDSTONE_WIRE)
}

pub(super) async fn test_oak_button_30_tick_pulse(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let btn_pos = BlockPos::new(1, 1, 1);
    let wire_pos = BlockPos::new(1, 1, 2);
    let mut btn = LeverLikeProperties::default(&Block::OAK_BUTTON);
    btn.face = AttachFace::Floor;
    btn.facing = HorizontalFacing::South;
    btn.powered = false;
    test.set_block(&btn_pos, btn.to_state_id(&Block::OAK_BUTTON))
        .await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    test.press_button(&btn_pos).await?;

    test.start_sequence()
        .then_idle(22)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await)
                .power;
            test.assert_true(
                p == 15,
                format!("Expected wooden button (30t) to remain powered at tick 22, got {p}"),
            )
        })
        .then_idle(12)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await)
                .power;
            test.assert_true(
                p == 0,
                format!("Expected wooden button to turn off after 30 ticks, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_polished_blackstone_button_20_tick_pulse(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let btn_pos = BlockPos::new(1, 1, 1);
    let wire_pos = BlockPos::new(1, 1, 2);
    let mut btn = LeverLikeProperties::default(&Block::POLISHED_BLACKSTONE_BUTTON);
    btn.face = AttachFace::Floor;
    btn.facing = HorizontalFacing::South;
    btn.powered = false;
    test.set_block(
        &btn_pos,
        btn.to_state_id(&Block::POLISHED_BLACKSTONE_BUTTON),
    )
    .await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    test.press_button(&btn_pos).await?;

    test.start_sequence()
        .then_idle(10)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await)
                .power;
            test.assert_true(p == 15, "Expected blackstone button powered at tick 10")
        })
        .then_idle(24)
        .then_execute(move |test| async move {
            let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await)
                .power;
            test.assert_true(p == 0, "Expected blackstone button unpowered after pulse")
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_stone_button_floor_mount_strong_power(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let support = BlockPos::new(2, 2, 2);
    let btn_pos = BlockPos::new(2, 3, 2);
    let wire_pos = BlockPos::new(2, 2, 3);
    test.set_block_default(&BlockPos::new(2, 1, 3), &Block::STONE)
        .await?;
    test.set_block_default(&support, &Block::STONE).await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    let mut btn = LeverLikeProperties::default(&Block::STONE_BUTTON);
    btn.face = AttachFace::Floor;
    btn.facing = HorizontalFacing::North;
    test.set_block(&btn_pos, btn.to_state_id(&Block::STONE_BUTTON))
        .await?;

    test.press_button(&btn_pos).await?;
    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await).power;
    test.assert_true(
        p == 15,
        format!("Expected floor-mounted button to strongly power block below (wire=15), got {p}"),
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_stone_button_ceiling_mount_strong_power(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let ceiling = BlockPos::new(2, 2, 2);
    let btn_pos = BlockPos::new(2, 1, 2);
    let wire_pos = BlockPos::new(2, 3, 2);
    test.set_block_default(&ceiling, &Block::STONE).await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    let mut btn = LeverLikeProperties::default(&Block::STONE_BUTTON);
    btn.face = AttachFace::Ceiling;
    btn.facing = HorizontalFacing::North;
    test.set_block(&btn_pos, btn.to_state_id(&Block::STONE_BUTTON))
        .await?;

    test.press_button(&btn_pos).await?;
    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await).power;
    test.assert_true(
        p == 15,
        format!("Expected ceiling-mounted button to strongly power block above (wire=15), got {p}"),
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_stone_button_wall_mount_strong_power(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    let btn_pos = BlockPos::new(2, 1, 1);
    let wire_pos = BlockPos::new(2, 1, 3);
    test.set_block_default(&wall, &Block::STONE).await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    let mut btn = LeverLikeProperties::default(&Block::STONE_BUTTON);
    btn.face = AttachFace::Wall;
    btn.facing = HorizontalFacing::North;
    test.set_block(&btn_pos, btn.to_state_id(&Block::STONE_BUTTON))
        .await?;

    test.press_button(&btn_pos).await?;
    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await).power;
    test.assert_true(
        p == 15,
        format!("Expected wall-mounted button to strongly power wall block (wire=15), got {p}"),
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_button_breaks_when_support_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    let btn_pos = BlockPos::new(2, 1, 1);
    test.set_block_default(&wall, &Block::STONE).await?;

    let mut btn = LeverLikeProperties::default(&Block::STONE_BUTTON);
    btn.face = AttachFace::Wall;
    btn.facing = HorizontalFacing::North;
    test.set_block(&btn_pos, btn.to_state_id(&Block::STONE_BUTTON))
        .await?;

    test.set_block_default(&wall, &Block::AIR).await?;
    test.assert_block_present(&btn_pos, &Block::AIR).await?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_lever_wall_mount_strong_power(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    let lever_pos = BlockPos::new(2, 1, 1);
    let wire_pos = BlockPos::new(2, 1, 3);
    test.set_block_default(&wall, &Block::STONE).await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    let mut lever = LeverLikeProperties::default(&Block::LEVER);
    lever.face = AttachFace::Wall;
    lever.facing = HorizontalFacing::North;
    lever.powered = false;
    test.set_block(&lever_pos, lever.to_state_id(&Block::LEVER))
        .await?;

    test.pull_lever(&lever_pos).await?;
    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await).power;
    test.assert_true(
        p == 15,
        format!("Expected wall lever to strongly power wall and wire to 15, got {p}"),
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_lever_ceiling_mount_strong_power(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let ceiling = BlockPos::new(2, 2, 2);
    let lever_pos = BlockPos::new(2, 1, 2);
    let wire_pos = BlockPos::new(2, 3, 2);
    test.set_block_default(&ceiling, &Block::STONE).await?;
    test.set_block(&wire_pos, ns_wire()).await?;

    let mut lever = LeverLikeProperties::default(&Block::LEVER);
    lever.face = AttachFace::Ceiling;
    lever.facing = HorizontalFacing::North;
    lever.powered = false;
    test.set_block(&lever_pos, lever.to_state_id(&Block::LEVER))
        .await?;

    test.pull_lever(&lever_pos).await?;
    let p = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await).power;
    test.assert_true(
        p == 15,
        format!("Expected ceiling lever to strongly power block above to 15, got {p}"),
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_lever_breaks_when_support_removed(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    let lever_pos = BlockPos::new(2, 1, 1);
    test.set_block_default(&wall, &Block::STONE).await?;

    let mut lever = LeverLikeProperties::default(&Block::LEVER);
    lever.face = AttachFace::Wall;
    lever.facing = HorizontalFacing::North;
    test.set_block(&lever_pos, lever.to_state_id(&Block::LEVER))
        .await?;

    test.set_block_default(&wall, &Block::AIR).await?;
    test.assert_block_present(&lever_pos, &Block::AIR).await?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_weighted_pressure_plate_light_scaling(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let plate_pos = BlockPos::new(2, 1, 2);
    test.set_block_default(&plate_pos, &Block::LIGHT_WEIGHTED_PRESSURE_PLATE)
        .await?;

    let pos = Vector3::new(2.5, 1.1, 2.5);
    test.spawn_item_at("minecraft:stone", 1, pos).await?;
    test.spawn_item_at("minecraft:dirt", 1, pos).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| async move {
            let p = LightWeightedPressurePlateLikeProperties::from_state_id(
                test.block_state_id(&plate_pos).await,
            )
            .power;
            test.assert_true(
                p == 2,
                format!("Expected light weighted pressure plate power 2 for 2 items, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_weighted_pressure_plate_heavy_scaling(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let plate_pos = BlockPos::new(2, 1, 2);
    test.set_block_default(&plate_pos, &Block::HEAVY_WEIGHTED_PRESSURE_PLATE)
        .await?;

    let pos = Vector3::new(2.5, 1.1, 2.5);
    test.spawn_item_at("minecraft:stone", 1, pos).await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| async move {
            let p = LightWeightedPressurePlateLikeProperties::from_state_id(
                test.block_state_id(&plate_pos).await,
            )
            .power;
            test.assert_true(
                p == 1,
                format!("Expected heavy weighted pressure plate power 1 for 1 entity, got {p}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_wooden_pressure_plate_item_activation(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 4, 4).await?;
    let oak_plate = BlockPos::new(2, 1, 2);
    test.set_block_default(&oak_plate, &Block::OAK_PRESSURE_PLATE)
        .await?;

    test.spawn_item_at("minecraft:apple", 1, Vector3::new(2.5, 1.1, 2.5))
        .await?;

    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| async move {
            let powered = StonePressurePlateLikeProperties::from_state_id(
                test.block_state_id(&oak_plate).await,
            )
            .powered;
            test.assert_true(
                powered,
                "Expected oak_pressure_plate to become powered when an item entity lands on it",
            )
        })
        .then_succeed();
    Ok(())
}
