use pumpkin_data::{
    Block,
    block_properties::{
        BeeNestLikeProperties, DispenserLikeProperties, Facing, RespawnAnchorLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_dispenser_honeycomb_beehive_and_shulker(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 6).await?;

    let mut east_dispenser = DispenserLikeProperties::default(&Block::DISPENSER);
    east_dispenser.facing = Facing::East;
    let dispenser_state = east_dispenser.to_state_id(&Block::DISPENSER);

    // 1. Dispenser waxes a copper block using honeycomb
    let wax_dispenser = BlockPos::new(1, 1, 1);
    let copper_pos = BlockPos::new(2, 1, 1);
    test.set_block(&wax_dispenser, dispenser_state).await?;
    test.set_block_default(&copper_pos, &Block::COPPER_BLOCK)
        .await?;
    test.set_container_item(&wax_dispenser, 0, "minecraft:honeycomb", 1)
        .await?;

    // 2. Dispenser harvests honey bottle from a full beehive (honey_level = 5)
    let hive_dispenser = BlockPos::new(1, 1, 2);
    let hive_pos = BlockPos::new(2, 1, 2);
    test.set_block(&hive_dispenser, dispenser_state).await?;
    let mut hive_props = BeeNestLikeProperties::default(&Block::BEEHIVE);
    hive_props.honey_level = 5;
    test.set_block(&hive_pos, hive_props.to_state_id(&Block::BEEHIVE))
        .await?;
    test.set_container_item(&hive_dispenser, 0, "minecraft:glass_bottle", 1)
        .await?;

    // 3. Dispenser charges a respawn anchor using glowstone
    let anchor_dispenser = BlockPos::new(1, 1, 3);
    let anchor_pos = BlockPos::new(2, 1, 3);
    test.set_block(&anchor_dispenser, dispenser_state).await?;
    test.set_block_default(&anchor_pos, &Block::RESPAWN_ANCHOR)
        .await?;
    test.set_container_item(&anchor_dispenser, 0, "minecraft:glowstone", 1)
        .await?;

    // 4. Dispenser places a shulker box into air
    let shulker_dispenser = BlockPos::new(1, 1, 4);
    let shulker_pos = BlockPos::new(2, 1, 4);
    test.set_block(&shulker_dispenser, dispenser_state).await?;
    test.set_container_item(&shulker_dispenser, 0, "minecraft:shulker_box", 1)
        .await?;

    // Trigger all four dispensers simultaneously
    for z in 1..=4 {
        test.set_block_default(&BlockPos::new(1, 2, z), &Block::REDSTONE_BLOCK)
            .await?;
    }

    test.start_sequence()
        .then_idle(6)
        .then_execute(move |test| async move {
            test.assert_block_present(&copper_pos, &Block::WAXED_COPPER_BLOCK)
                .await?;
            test.assert_container_empty(&wax_dispenser).await?;

            let hive_state = test.block_state_id(&hive_pos).await;
            let hive_after = BeeNestLikeProperties::from_state_id(hive_state);
            test.assert_true(
                hive_after.honey_level == 0,
                format!(
                    "expected beehive honey_level 0 after dispenser bottle harvest, found {}",
                    hive_after.honey_level
                ),
            )?;
            test.assert_container_contains_count(&hive_dispenser, "minecraft:honey_bottle", 1)
                .await?;

            let anchor_state = test.block_state_id(&anchor_pos).await;
            let anchor_after = RespawnAnchorLikeProperties::from_state_id(anchor_state);
            test.assert_true(
                anchor_after.charges == 1,
                format!(
                    "expected respawn anchor charges == 1 after glowstone dispense, found {}",
                    anchor_after.charges
                ),
            )?;
            test.assert_container_empty(&anchor_dispenser).await?;

            test.assert_block_present(&shulker_pos, &Block::SHULKER_BOX)
                .await?;
            test.assert_container_empty(&shulker_dispenser).await
        })
        .then_succeed();
    Ok(())
}
