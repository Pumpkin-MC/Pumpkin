use pumpkin_data::{Block, block_properties::FurnaceLikeProperties};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

pub(super) async fn test_furnace_ignites_and_smelts(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;

    let furnace_pos = BlockPos::new(1, 1, 1);
    test.set_block_default(&furnace_pos, &Block::FURNACE)
        .await?;

    // Slot 0 = input (raw_iron), Slot 1 = fuel (coal)
    test.set_container_item(&furnace_pos, 0, "minecraft:raw_iron", 1)
        .await?;
    test.set_container_item(&furnace_pos, 1, "minecraft:coal", 1)
        .await?;

    test.succeed_when(move |test| async move {
        let props = FurnaceLikeProperties::from_state_id(test.block_state_id(&furnace_pos).await);
        test.assert_true(
            props.lit,
            "Expected furnace to ignite (lit=true) when loaded with raw_iron and coal",
        )?;
        test.assert_container_contains_count(&furnace_pos, "minecraft:coal", 0)
            .await
    })?;
    Ok(())
}

pub(super) async fn test_furnace_returns_empty_bucket_from_lava_bucket_fuel(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;

    let furnace_pos = BlockPos::new(1, 1, 1);
    test.set_block_default(&furnace_pos, &Block::FURNACE)
        .await?;
    test.set_container_item(&furnace_pos, 0, "minecraft:raw_iron", 1)
        .await?;
    test.set_container_item(&furnace_pos, 1, "minecraft:lava_bucket", 1)
        .await?;

    test.start_sequence()
        .then_wait_until(move |test| async move {
            test.assert_container_contains_count(&furnace_pos, "minecraft:lava_bucket", 0)
                .await?;
            test.assert_container_contains_count(&furnace_pos, "minecraft:bucket", 1)
                .await
        })
        .then_idle(1)
        .then_execute(move |test| async move {
            test.assert_container_contains_count(&furnace_pos, "minecraft:bucket", 1)
                .await
        })
        .then_succeed();
    Ok(())
}
