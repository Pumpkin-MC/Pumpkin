use pumpkin_data::{Block, block_properties::ScaffoldingLikeProperties};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_scaffolding_distance_and_collapse(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let column_base = BlockPos::new(1, 1, 2);
    let column_top = BlockPos::new(1, 2, 2);
    let overhang_1 = BlockPos::new(2, 2, 2);

    let mut base_props = ScaffoldingLikeProperties::default(&Block::SCAFFOLDING);
    base_props.distance = 0;
    base_props.bottom = false;
    helper
        .set_block(&column_base, base_props.to_state_id(&Block::SCAFFOLDING))
        .await?;
    helper
        .set_block(&column_top, base_props.to_state_id(&Block::SCAFFOLDING))
        .await?;

    let mut overhang_props = ScaffoldingLikeProperties::default(&Block::SCAFFOLDING);
    overhang_props.distance = 1;
    overhang_props.bottom = true;
    helper
        .set_block(&overhang_1, overhang_props.to_state_id(&Block::SCAFFOLDING))
        .await?;

    // Removing the supporting top block causes the overhang's neighbor update to schedule a collapse to AIR (distance == 7)
    helper.set_block_default(&column_top, &Block::AIR).await?;

    helper.succeed_when(move |helper| async move {
        helper.assert_block_present(&overhang_1, &Block::AIR).await
    })
}
