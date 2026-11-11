use pumpkin_data::{Block, block_properties::FurnaceLikeProperties};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_furnace_ignites_and_smelts(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 3, 3).await?;

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
