use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

pub(super) async fn test_brewing_returns_glass_bottle_from_dragon_breath(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;

    let stand_pos = BlockPos::new(1, 1, 1);
    test.set_block_default(&stand_pos, &Block::BREWING_STAND)
        .await?;

    test
        .execute_command(
            &stand_pos,
            "item replace block ~ ~ ~ container.0 with minecraft:splash_potion[minecraft:potion_contents={potion:\"minecraft:water\"}]",
        )
        .await?;
    test.set_container_item(&stand_pos, 4, "minecraft:blaze_powder", 1)
        .await?;
    test.set_container_item(&stand_pos, 3, "minecraft:dragon_breath", 1)
        .await?;

    test.start_sequence()
        .then_idle(410)
        .then_execute(move |test| async move {
            test.assert_container_contains_count(&stand_pos, "minecraft:lingering_potion", 1)
                .await?;
            test.assert_container_contains_count(&stand_pos, "minecraft:dragon_breath", 0)
                .await?;
            test.assert_container_contains_count(&stand_pos, "minecraft:blaze_powder", 0)
                .await?;
            test.assert_container_contains_count(&stand_pos, "minecraft:glass_bottle", 1)
                .await
        })
        .then_succeed();
    Ok(())
}
