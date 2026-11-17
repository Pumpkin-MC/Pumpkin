use pumpkin_data::{
    Block,
    block_properties::{
        AttachFace, ComparatorLikeProperties, EastRedstone, HorizontalFacing, LeverLikeProperties,
        ModeComparator, NorthRedstone, RedstoneOreLikeProperties, RedstoneWireLikeProperties,
        RepeaterLikeProperties, SouthRedstone, WaterCauldronLikeProperties, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_redstone_torch_inverter(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let support_pos = BlockPos::new(2, 1, 2);
    let torch_pos = BlockPos::new(2, 2, 2);
    let lamp_pos = BlockPos::new(2, 3, 2);
    let lever_pos = BlockPos::new(2, 1, 1);

    helper
        .set_block_default(&support_pos, &Block::STONE)
        .await?;
    helper
        .set_block_default(&lamp_pos, &Block::REDSTONE_LAMP)
        .await?;
    helper
        .set_block_default(&torch_pos, &Block::REDSTONE_TORCH)
        .await?;

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Wall;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    helper
        .set_block(&lever_pos, lever_props.to_state_id(&Block::LEVER))
        .await?;

    let lit_lamp = {
        let mut props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
        props.lit = true;
        props.to_state_id(&Block::REDSTONE_LAMP)
    };
    let unlit_lamp = {
        let mut props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
        props.lit = false;
        props.to_state_id(&Block::REDSTONE_LAMP)
    };
    let lit_torch = {
        let mut props = RedstoneOreLikeProperties::default(&Block::REDSTONE_TORCH);
        props.lit = true;
        props.to_state_id(&Block::REDSTONE_TORCH)
    };
    let unlit_torch = {
        let mut props = RedstoneOreLikeProperties::default(&Block::REDSTONE_TORCH);
        props.lit = false;
        props.to_state_id(&Block::REDSTONE_TORCH)
    };

    helper.assert_block_state(&torch_pos, lit_torch).await?;
    helper.assert_block_state(&lamp_pos, lit_lamp).await?;

    helper.pull_lever(&lever_pos).await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&torch_pos, unlit_torch).await
        })
        .then_idle(5)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&lamp_pos, unlit_lamp).await?;
            helper.pull_lever(&lever_pos).await
        })
        .then_idle(3)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&torch_pos, lit_torch).await?;
            helper.assert_block_state(&lamp_pos, lit_lamp).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_repeater_delay(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let input_pos = BlockPos::new(2, 1, 1);
    let repeater_pos = BlockPos::new(2, 1, 2);
    let lamp_pos = BlockPos::new(2, 1, 3);

    let mut repeater_props = RepeaterLikeProperties::default(&Block::REPEATER);
    repeater_props.facing = HorizontalFacing::North;
    repeater_props.delay = 3;
    repeater_props.powered = false;
    let unpowered_repeater = repeater_props.to_state_id(&Block::REPEATER);
    repeater_props.powered = true;
    let powered_repeater = repeater_props.to_state_id(&Block::REPEATER);

    let mut lamp_props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
    lamp_props.lit = false;
    let unlit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);
    lamp_props.lit = true;
    let lit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);

    helper.set_block(&repeater_pos, unpowered_repeater).await?;
    helper.set_block(&lamp_pos, unlit_lamp).await?;
    helper
        .set_block_default(&input_pos, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            helper
                .assert_block_state(&repeater_pos, unpowered_repeater)
                .await?;
            helper.assert_block_state(&lamp_pos, unlit_lamp).await
        })
        .then_idle(5)
        .then_execute(move |helper| async move {
            helper
                .assert_block_state(&repeater_pos, powered_repeater)
                .await?;
            helper.assert_block_state(&lamp_pos, lit_lamp).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_redstone_comparator_modes(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let chest_pos = BlockPos::new(2, 1, 1);
    let main_comp_pos = BlockPos::new(2, 1, 2);
    let wire_pos = BlockPos::new(2, 1, 3);
    let cauldron_pos = BlockPos::new(0, 1, 2);
    let side_comp_pos = BlockPos::new(1, 1, 2);

    helper.set_block_default(&chest_pos, &Block::CHEST).await?;
    for slot in 0..13 {
        helper
            .set_container_item(&chest_pos, slot, "minecraft:cobblestone", 64)
            .await?;
    }

    let mut cauldron_props = WaterCauldronLikeProperties::default(&Block::WATER_CAULDRON);
    cauldron_props.level = 3;
    helper
        .set_block(
            &cauldron_pos,
            cauldron_props.to_state_id(&Block::WATER_CAULDRON),
        )
        .await?;

    let mut side_comp = ComparatorLikeProperties::default(&Block::COMPARATOR);
    side_comp.facing = HorizontalFacing::West;
    side_comp.mode = ModeComparator::Compare;
    helper
        .set_block(&side_comp_pos, side_comp.to_state_id(&Block::COMPARATOR))
        .await?;

    let mut wire_props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    wire_props.north = NorthRedstone::Side;
    wire_props.south = SouthRedstone::Side;
    wire_props.east = EastRedstone::None;
    wire_props.west = WestRedstone::None;
    helper
        .set_block(&wire_pos, wire_props.to_state_id(&Block::REDSTONE_WIRE))
        .await?;

    let mut main_comp = ComparatorLikeProperties::default(&Block::COMPARATOR);
    main_comp.facing = HorizontalFacing::North;
    main_comp.mode = ModeComparator::Compare;
    helper
        .set_block(&main_comp_pos, main_comp.to_state_id(&Block::COMPARATOR))
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            let wire_state = helper.block_state_id(&wire_pos).await;
            let wire_power = RedstoneWireLikeProperties::from_state_id(wire_state).power;
            helper.assert_true(
                wire_power == 7,
                format!("Expected compare-mode wire power 7, got {wire_power}"),
            )?;

            let mut subtract_comp = ComparatorLikeProperties::from_state_id(
                helper.block_state_id(&main_comp_pos).await,
            );
            subtract_comp.mode = ModeComparator::Subtract;
            helper
                .set_block(
                    &main_comp_pos,
                    subtract_comp.to_state_id(&Block::COMPARATOR),
                )
                .await?;
            helper
                .set_block_default(&BlockPos::new(3, 1, 2), &Block::STONE)
                .await
        })
        .then_idle(4)
        .then_execute(move |helper| async move {
            let wire_state = helper.block_state_id(&wire_pos).await;
            let wire_power = RedstoneWireLikeProperties::from_state_id(wire_state).power;
            helper.assert_true(
                wire_power == 4,
                format!("Expected subtract-mode wire power 4 (7 - 3), got {wire_power}"),
            )
        })
        .then_succeed();
    Ok(())
}
