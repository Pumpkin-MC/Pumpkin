use pumpkin_data::{Block, block_properties::FurnaceLikeProperties};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_furnace_ignites_and_smelts(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;

    let furnace_pos = BlockPos::new(1, 1, 1);
    helper
        .set_block_default(&furnace_pos, &Block::FURNACE)
        .await?;

    // Slot 0 = input (raw_iron), Slot 1 = fuel (coal)
    helper
        .set_container_item(&furnace_pos, 0, "minecraft:raw_iron", 1)
        .await?;
    helper
        .set_container_item(&furnace_pos, 1, "minecraft:coal", 1)
        .await?;

    helper.succeed_when(move |helper| async move {
        let props = FurnaceLikeProperties::from_state_id(helper.block_state_id(&furnace_pos).await);
        helper.assert_true(
            props.lit,
            "Expected furnace to ignite (lit=true) when loaded with raw_iron and coal",
        )?;
        helper
            .assert_container_contains_count(&furnace_pos, "minecraft:coal", 0)
            .await
    })?;
    Ok(())
}
