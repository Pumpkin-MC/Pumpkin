use pumpkin_data::{
    Block,
    block_properties::{CrafterLikeProperties, DispenserLikeProperties, Facing, Orientation},
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

pub(super) async fn test_dispenser_and_dropper(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(6, 4, 6).await?;
    test.build_floor(6, 6, &Block::STONE).await?;

    let dropper_pos = BlockPos::new(1, 1, 1);
    let chest_pos = BlockPos::new(2, 1, 1);
    let dropper_power = BlockPos::new(1, 2, 1);

    let mut dropper_props = DispenserLikeProperties::default(&Block::DROPPER);
    dropper_props.facing = Facing::East;
    test.set_block(&dropper_pos, dropper_props.to_state_id(&Block::DROPPER))
        .await?;
    test.set_block_default(&chest_pos, &Block::CHEST).await?;
    test.set_container_item(&dropper_pos, 0, "minecraft:emerald", 3)
        .await?;

    let dispenser_pos = BlockPos::new(1, 1, 4);
    let water_target = BlockPos::new(2, 1, 4);
    let dispenser_power = BlockPos::new(1, 2, 4);

    let mut dispenser_props = DispenserLikeProperties::default(&Block::DISPENSER);
    dispenser_props.facing = Facing::East;
    test.set_block(
        &dispenser_pos,
        dispenser_props.to_state_id(&Block::DISPENSER),
    )
    .await?;
    for offset in [
        BlockPos::new(3, 1, 4),
        BlockPos::new(2, 1, 3),
        BlockPos::new(2, 1, 5),
    ] {
        test.set_block_default(&offset, &Block::STONE).await?;
    }
    test.set_container_item(&dispenser_pos, 0, "minecraft:water_bucket", 1)
        .await?;

    test.set_block_default(&dropper_power, &Block::REDSTONE_BLOCK)
        .await?;
    test.set_block_default(&dispenser_power, &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            test.assert_container_contains_count(&chest_pos, "minecraft:emerald", 1)
                .await?;
            test.assert_container_contains_count(&dropper_pos, "minecraft:emerald", 2)
                .await?;
            test.assert_block_present(&water_target, &Block::WATER)
                .await?;
            test.assert_container_contains_count(&dispenser_pos, "minecraft:bucket", 1)
                .await?;

            test.set_block_default(&dispenser_power, &Block::AIR).await
        })
        .then_idle(2)
        .then_execute(move |test| async move {
            test.set_block_default(&dispenser_power, &Block::REDSTONE_BLOCK)
                .await
        })
        .then_idle(6)
        .then_execute(move |test| async move {
            test.assert_block_present(&water_target, &Block::AIR)
                .await?;
            test.assert_container_contains_count(&dispenser_pos, "minecraft:water_bucket", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_crafter_crafts_into_chest(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let crafter_pos = BlockPos::new(1, 1, 2);
    let chest_pos = BlockPos::new(2, 1, 2);
    let power_pos = BlockPos::new(1, 2, 2);

    let mut crafter_props = CrafterLikeProperties::default(&Block::CRAFTER);
    crafter_props.orientation = Orientation::EastUp;
    crafter_props.triggered = false;
    crafter_props.crafting = false;
    test.set_block(&crafter_pos, crafter_props.to_state_id(&Block::CRAFTER))
        .await?;
    test.set_block_default(&chest_pos, &Block::CHEST).await?;

    test.set_container_item(&crafter_pos, 0, "minecraft:oak_log", 1)
        .await?;

    test.set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            test.assert_container_empty(&crafter_pos).await?;
            test.assert_container_contains_count(&chest_pos, "minecraft:oak_planks", 4)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_crafter_dispenses_crafting_remainders(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;

    let crafter_pos = BlockPos::new(1, 1, 1);
    let chest_pos = crafter_pos.up();
    let mut props = CrafterLikeProperties::default(&Block::CRAFTER);
    props.orientation = Orientation::UpNorth;
    test.set_block(&crafter_pos, props.to_state_id(&Block::CRAFTER))
        .await?;
    test.set_block_default(&chest_pos, &Block::CHEST).await?;

    for slot in [0, 1, 3, 4] {
        test.set_container_item(&crafter_pos, slot, "minecraft:honey_bottle", 1)
            .await?;
    }
    test.set_block_default(&crafter_pos.down(), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            test.assert_container_empty(&crafter_pos).await?;
            test.assert_container_contains_count(&chest_pos, "minecraft:honey_block", 1)
                .await?;
            test.assert_container_contains_count(&chest_pos, "minecraft:glass_bottle", 4)
                .await
        })
        .then_succeed();
    Ok(())
}
