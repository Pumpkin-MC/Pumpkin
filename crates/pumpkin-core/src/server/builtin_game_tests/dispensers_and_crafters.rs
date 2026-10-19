use pumpkin_data::{
    Block,
    block_properties::{CrafterLikeProperties, DispenserLikeProperties, Facing, Orientation},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_dispenser_and_dropper(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 6).await?;

    let dropper_pos = BlockPos::new(1, 1, 1);
    let chest_pos = BlockPos::new(2, 1, 1);
    let dropper_power = BlockPos::new(1, 2, 1);

    let mut dropper_props = DispenserLikeProperties::default(&Block::DROPPER);
    dropper_props.facing = Facing::East;
    helper
        .set_block(&dropper_pos, dropper_props.to_state_id(&Block::DROPPER))
        .await?;
    helper.set_block_default(&chest_pos, &Block::CHEST).await?;
    helper
        .set_container_item(&dropper_pos, 0, "minecraft:emerald", 3)
        .await?;

    let dispenser_pos = BlockPos::new(1, 1, 4);
    let water_target = BlockPos::new(2, 1, 4);
    let dispenser_power = BlockPos::new(1, 2, 4);

    let mut dispenser_props = DispenserLikeProperties::default(&Block::DISPENSER);
    dispenser_props.facing = Facing::East;
    helper
        .set_block(
            &dispenser_pos,
            dispenser_props.to_state_id(&Block::DISPENSER),
        )
        .await?;
    for offset in [
        BlockPos::new(3, 1, 4),
        BlockPos::new(2, 1, 3),
        BlockPos::new(2, 1, 5),
    ] {
        helper.set_block_default(&offset, &Block::STONE).await?;
    }
    helper
        .set_container_item(&dispenser_pos, 0, "minecraft:water_bucket", 1)
        .await?;

    helper
        .set_block_default(&dropper_power, &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&dispenser_power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&chest_pos, "minecraft:emerald", 1)
                .await?;
            helper
                .assert_container_contains_count(&dropper_pos, "minecraft:emerald", 2)
                .await?;
            helper
                .assert_block_present(&water_target, &Block::WATER)
                .await?;
            helper
                .assert_container_contains_count(&dispenser_pos, "minecraft:bucket", 1)
                .await?;

            helper
                .set_block_default(&dispenser_power, &Block::AIR)
                .await
        })
        .then_idle(2)
        .then_execute(move |helper| async move {
            helper
                .set_block_default(&dispenser_power, &Block::REDSTONE_BLOCK)
                .await
        })
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&water_target, &Block::AIR)
                .await?;
            helper
                .assert_container_contains_count(&dispenser_pos, "minecraft:water_bucket", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_crafter_crafts_into_chest(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let crafter_pos = BlockPos::new(1, 1, 2);
    let chest_pos = BlockPos::new(2, 1, 2);
    let power_pos = BlockPos::new(1, 2, 2);

    let mut crafter_props = CrafterLikeProperties::default(&Block::CRAFTER);
    crafter_props.orientation = Orientation::EastUp;
    crafter_props.triggered = false;
    crafter_props.crafting = false;
    helper
        .set_block(&crafter_pos, crafter_props.to_state_id(&Block::CRAFTER))
        .await?;
    helper.set_block_default(&chest_pos, &Block::CHEST).await?;

    helper
        .set_container_item(&crafter_pos, 0, "minecraft:oak_log", 1)
        .await?;

    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper.assert_container_empty(&crafter_pos).await?;
            helper
                .assert_container_contains_count(&chest_pos, "minecraft:oak_planks", 4)
                .await
        })
        .then_succeed();
    Ok(())
}
