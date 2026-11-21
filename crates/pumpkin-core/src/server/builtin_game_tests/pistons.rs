use pumpkin_data::{
    Block,
    block_properties::{Facing, StickyPistonLikeProperties},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_piston_and_sticky_piston(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 6).await?;

    let piston_pos = BlockPos::new(1, 1, 1);
    let piston_front = BlockPos::new(2, 1, 1);
    let piston_pushed = BlockPos::new(3, 1, 1);
    let piston_power = BlockPos::new(1, 2, 1);

    let sticky_pos = BlockPos::new(1, 1, 4);
    let sticky_front = BlockPos::new(2, 1, 4);
    let sticky_pushed = BlockPos::new(3, 1, 4);
    let sticky_power = BlockPos::new(1, 2, 4);

    let mut piston_props = StickyPistonLikeProperties::default(&Block::PISTON);
    piston_props.facing = Facing::East;
    piston_props.extended = false;
    helper
        .set_block(&piston_pos, piston_props.to_state_id(&Block::PISTON))
        .await?;
    helper
        .set_block_default(&piston_front, &Block::GOLD_BLOCK)
        .await?;

    let mut sticky_props = StickyPistonLikeProperties::default(&Block::STICKY_PISTON);
    sticky_props.facing = Facing::East;
    sticky_props.extended = false;
    helper
        .set_block(&sticky_pos, sticky_props.to_state_id(&Block::STICKY_PISTON))
        .await?;
    helper
        .set_block_default(&sticky_front, &Block::DIAMOND_BLOCK)
        .await?;

    helper
        .set_block_default(&piston_power, &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&sticky_power, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&piston_front, &Block::PISTON_HEAD)
                .await?;
            helper
                .assert_block_present(&piston_pushed, &Block::GOLD_BLOCK)
                .await?;
            helper
                .assert_block_present(&sticky_front, &Block::PISTON_HEAD)
                .await?;
            helper
                .assert_block_present(&sticky_pushed, &Block::DIAMOND_BLOCK)
                .await?;

            helper.set_block_default(&piston_power, &Block::AIR).await?;
            helper.set_block_default(&sticky_power, &Block::AIR).await
        })
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&piston_front, &Block::AIR)
                .await?;
            helper
                .assert_block_present(&piston_pushed, &Block::GOLD_BLOCK)
                .await?;
            helper
                .assert_block_present(&sticky_front, &Block::DIAMOND_BLOCK)
                .await?;
            helper
                .assert_block_present(&sticky_pushed, &Block::AIR)
                .await
        })
        .then_succeed();
    Ok(())
}
