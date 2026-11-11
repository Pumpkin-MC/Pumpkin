use pumpkin_data::{
    Block,
    block_properties::{
        OakDoorLikeProperties, OakFenceGateLikeProperties, OakTrapdoorLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_doors_trapdoors_fence_gates_redstone(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 7, 4).await?;

    let door_lower = BlockPos::new(1, 1, 1);
    let door_upper = BlockPos::new(1, 2, 1);
    let door_power = BlockPos::new(1, 1, 2);

    let trapdoor_pos = BlockPos::new(3, 1, 1);
    let trapdoor_power = BlockPos::new(3, 1, 2);

    let gate_pos = BlockPos::new(5, 1, 1);
    let gate_power = BlockPos::new(5, 1, 2);

    test.set_block_default(&door_lower, &Block::OAK_DOOR)
        .await?;
    test.set_block_default(&trapdoor_pos, &Block::IRON_TRAPDOOR)
        .await?;
    test.set_block_default(&gate_pos, &Block::OAK_FENCE_GATE)
        .await?;

    test.set_block_default(&door_power, &Block::REDSTONE_BLOCK)
        .await?;
    test.set_block_default(&trapdoor_power, &Block::REDSTONE_BLOCK)
        .await?;
    test.set_block_default(&gate_power, &Block::REDSTONE_BLOCK)
        .await?;

    let lower_props = OakDoorLikeProperties::from_state_id(test.block_state_id(&door_lower).await);
    let upper_props = OakDoorLikeProperties::from_state_id(test.block_state_id(&door_upper).await);
    let trap_props =
        OakTrapdoorLikeProperties::from_state_id(test.block_state_id(&trapdoor_pos).await);
    let gate_props =
        OakFenceGateLikeProperties::from_state_id(test.block_state_id(&gate_pos).await);

    test.assert_true(
        lower_props.open && lower_props.powered && upper_props.open && upper_props.powered,
        "Expected powered oak door halves to open",
    )?;
    test.assert_true(
        trap_props.open && trap_props.powered,
        "Expected powered iron trapdoor to open",
    )?;
    test.assert_true(
        gate_props.open && gate_props.powered,
        "Expected powered oak fence gate to open",
    )?;

    test.set_block_default(&door_power, &Block::AIR).await?;
    test.set_block_default(&trapdoor_power, &Block::AIR).await?;
    test.set_block_default(&gate_power, &Block::AIR).await?;

    let lower_props = OakDoorLikeProperties::from_state_id(test.block_state_id(&door_lower).await);
    let trap_props =
        OakTrapdoorLikeProperties::from_state_id(test.block_state_id(&trapdoor_pos).await);
    let gate_props =
        OakFenceGateLikeProperties::from_state_id(test.block_state_id(&gate_pos).await);

    test.assert_true(
        !lower_props.open && !lower_props.powered,
        "Expected unpowered oak door to close",
    )?;
    test.assert_true(
        !trap_props.open && !trap_props.powered,
        "Expected unpowered iron trapdoor to close",
    )?;
    test.assert_true(
        !gate_props.open && !gate_props.powered,
        "Expected unpowered oak fence gate to close",
    )?;

    test.succeed();
    Ok(())
}
