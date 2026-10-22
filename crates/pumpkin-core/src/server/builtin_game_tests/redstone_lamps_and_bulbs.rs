use pumpkin_data::{
    Block,
    block_properties::{
        ComparatorLikeProperties, CopperBulbLikeProperties, EastRedstone, HorizontalFacing,
        ModeComparator, NorthRedstone, RedstoneOreLikeProperties, RedstoneWireLikeProperties,
        SouthRedstone, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_copper_bulb_toggles_and_comparator(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let power_pos = BlockPos::new(2, 1, 1);
    let bulb_pos = BlockPos::new(2, 1, 2);
    let comp_pos = BlockPos::new(2, 1, 3);
    let wire_pos = BlockPos::new(2, 1, 4);

    helper
        .set_block_default(&bulb_pos, &Block::COPPER_BULB)
        .await?;

    let mut wire_props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    wire_props.north = NorthRedstone::Side;
    wire_props.south = SouthRedstone::Side;
    wire_props.east = EastRedstone::None;
    wire_props.west = WestRedstone::None;
    helper
        .set_block(&wire_pos, wire_props.to_state_id(&Block::REDSTONE_WIRE))
        .await?;

    let mut comp_props = ComparatorLikeProperties::default(&Block::COMPARATOR);
    comp_props.facing = HorizontalFacing::North;
    comp_props.mode = ModeComparator::Compare;
    helper
        .set_block(&comp_pos, comp_props.to_state_id(&Block::COMPARATOR))
        .await?;

    // 1. First rising edge: bulb becomes lit=true, powered=true
    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    let props = CopperBulbLikeProperties::from_state_id(helper.block_state_id(&bulb_pos).await);
    helper.assert_true(
        props.lit && props.powered,
        "Expected copper bulb to turn on and become powered on first rising edge",
    )?;

    // 2. Falling edge: bulb stays lit=true while powered becomes false
    helper.set_block_default(&power_pos, &Block::AIR).await?;
    let props = CopperBulbLikeProperties::from_state_id(helper.block_state_id(&bulb_pos).await);
    helper.assert_true(
        props.lit && !props.powered,
        "Expected copper bulb to stay lit when power is removed",
    )?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            let wire_power =
                RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await)
                    .power;
            helper.assert_true(
                wire_power == 15,
                format!("Expected comparator output 15 from lit copper bulb, got {wire_power}"),
            )?;

            // 3. Second rising edge: bulb toggles lit=false
            helper
                .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
                .await
        })
        .then_idle(3)
        .then_execute(move |helper| async move {
            let props =
                CopperBulbLikeProperties::from_state_id(helper.block_state_id(&bulb_pos).await);
            helper.assert_true(
                !props.lit && props.powered,
                "Expected copper bulb to toggle off on second rising edge",
            )?;
            let wire_power =
                RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await)
                    .power;
            helper.assert_true(
                wire_power == 0,
                format!("Expected comparator output 0 from unlit copper bulb, got {wire_power}"),
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_lamp_delayed_off(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let power_pos = BlockPos::new(2, 1, 1);
    let lamp_pos = BlockPos::new(2, 1, 2);

    let mut lamp_props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
    lamp_props.lit = false;
    let unlit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);
    lamp_props.lit = true;
    let lit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);

    helper.set_block(&lamp_pos, unlit_lamp).await?;
    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;
    helper.assert_block_state(&lamp_pos, lit_lamp).await?;

    // Removing power schedules a 4-tick turn-off delay
    helper.set_block_default(&power_pos, &Block::AIR).await?;
    helper.assert_block_state(&lamp_pos, lit_lamp).await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(
            move |helper| async move { helper.assert_block_state(&lamp_pos, lit_lamp).await },
        )
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&lamp_pos, unlit_lamp).await
        })
        .then_succeed();
    Ok(())
}
