use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_brewing_returns_glass_bottle_from_dragon_breath(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;

    let stand_pos = BlockPos::new(1, 1, 1);
    helper
        .set_block_default(&stand_pos, &Block::BREWING_STAND)
        .await?;

    helper
        .execute_command(
            &stand_pos,
            "item replace block ~ ~ ~ container.0 with minecraft:splash_potion[minecraft:potion_contents={potion:\"minecraft:water\"}]",
        )
        .await?;
    helper
        .set_container_item(&stand_pos, 4, "minecraft:blaze_powder", 1)
        .await?;
    helper
        .set_container_item(&stand_pos, 3, "minecraft:dragon_breath", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(410)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&stand_pos, "minecraft:lingering_potion", 1)
                .await?;
            helper
                .assert_container_contains_count(&stand_pos, "minecraft:dragon_breath", 0)
                .await?;
            helper
                .assert_container_contains_count(&stand_pos, "minecraft:blaze_powder", 0)
                .await?;
            helper
                .assert_container_contains_count(&stand_pos, "minecraft:glass_bottle", 1)
                .await
        })
        .then_succeed();
    Ok(())
}
