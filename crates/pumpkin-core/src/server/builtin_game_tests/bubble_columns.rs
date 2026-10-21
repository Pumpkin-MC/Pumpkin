use pumpkin_data::{Block, block_properties::BubbleColumnLikeProperties};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_bubble_column_upward_and_downward(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    for x in 0..5 {
        for z in 0..5 {
            if x == 0 || x == 4 || z == 0 || z == 4 || x == 2 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    let soul_sand_floor = BlockPos::new(1, 0, 2);
    let upward_col = BlockPos::new(1, 1, 2);
    let magma_floor = BlockPos::new(3, 0, 2);
    let downward_col = BlockPos::new(3, 1, 2);

    helper
        .set_block_default(&soul_sand_floor, &Block::SOUL_SAND)
        .await?;
    helper
        .set_block_default(&magma_floor, &Block::MAGMA_BLOCK)
        .await?;
    helper.set_block_default(&upward_col, &Block::WATER).await?;
    helper
        .set_block_default(&downward_col, &Block::WATER)
        .await?;

    let expected_upward =
        BubbleColumnLikeProperties { drag: false }.to_state_id(&Block::BUBBLE_COLUMN);
    let expected_downward =
        BubbleColumnLikeProperties { drag: true }.to_state_id(&Block::BUBBLE_COLUMN);

    helper
        .start_sequence()
        .then_idle(22)
        .then_execute(move |helper| async move {
            helper
                .assert_block_state(&upward_col, expected_upward)
                .await?;
            helper
                .assert_block_state(&downward_col, expected_downward)
                .await
        })
        .then_succeed();
    Ok(())
}
