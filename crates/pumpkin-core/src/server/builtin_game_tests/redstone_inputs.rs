use pumpkin_data::{
    Block,
    block_properties::{
        AttachFace, Facing, HorizontalFacing, LeverLikeProperties,
        LightWeightedPressurePlateLikeProperties, ObserverLikeProperties,
        RedstoneOreLikeProperties, RedstoneWireLikeProperties, StonePressurePlateLikeProperties,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_redstone_observer_pulse(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let target_pos = BlockPos::new(2, 1, 1);
    let observer_pos = BlockPos::new(2, 1, 2);
    let lamp_pos = BlockPos::new(2, 1, 3);

    let mut obs_props = ObserverLikeProperties::default(&Block::OBSERVER);
    obs_props.facing = Facing::North;
    obs_props.powered = false;
    let unpowered_obs = obs_props.to_state_id(&Block::OBSERVER);
    obs_props.powered = true;
    let powered_obs = obs_props.to_state_id(&Block::OBSERVER);

    let mut lamp_props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
    lamp_props.lit = false;
    let unlit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);
    lamp_props.lit = true;
    let lit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);

    helper.set_block(&observer_pos, unpowered_obs).await?;
    helper.set_block(&lamp_pos, unlit_lamp).await?;
    helper.set_block_default(&target_pos, &Block::STONE).await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            helper
                .assert_block_state(&observer_pos, powered_obs)
                .await?;
            helper.assert_block_state(&lamp_pos, lit_lamp).await
        })
        .then_idle(3)
        .then_execute(move |helper| async move {
            helper
                .assert_block_state(&observer_pos, unpowered_obs)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_stone_button_pulse(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let button_pos = BlockPos::new(2, 1, 1);
    let support_pos = BlockPos::new(2, 1, 2);
    let lamp_pos = BlockPos::new(2, 2, 2);

    helper
        .set_block_default(&support_pos, &Block::STONE)
        .await?;
    helper
        .set_block_default(&lamp_pos, &Block::REDSTONE_LAMP)
        .await?;

    let mut button_props = LeverLikeProperties::default(&Block::STONE_BUTTON);
    button_props.face = AttachFace::Wall;
    button_props.facing = HorizontalFacing::North;
    button_props.powered = false;
    let unpressed = button_props.to_state_id(&Block::STONE_BUTTON);
    button_props.powered = true;
    let pressed = button_props.to_state_id(&Block::STONE_BUTTON);

    let mut lamp_props = RedstoneOreLikeProperties::default(&Block::REDSTONE_LAMP);
    lamp_props.lit = false;
    let unlit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);
    lamp_props.lit = true;
    let lit_lamp = lamp_props.to_state_id(&Block::REDSTONE_LAMP);

    helper.set_block(&button_pos, unpressed).await?;
    helper.press_button(&button_pos).await?;

    helper.assert_block_state(&button_pos, pressed).await?;
    helper.assert_block_state(&lamp_pos, lit_lamp).await?;

    helper
        .start_sequence()
        .then_idle(21)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&button_pos, unpressed).await
        })
        .then_idle(5)
        .then_execute(move |helper| async move {
            helper.assert_block_state(&lamp_pos, unlit_lamp).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_lever_toggles_power(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let lever_pos = BlockPos::new(2, 1, 1);
    let wire_pos = BlockPos::new(2, 1, 2);
    let wire_far_pos = BlockPos::new(2, 1, 3);

    let mut lever_props = LeverLikeProperties::default(&Block::LEVER);
    lever_props.face = AttachFace::Floor;
    lever_props.facing = HorizontalFacing::North;
    lever_props.powered = false;
    let unpowered_lever = lever_props.to_state_id(&Block::LEVER);
    lever_props.powered = true;
    let powered_lever = lever_props.to_state_id(&Block::LEVER);

    helper.set_block(&lever_pos, unpowered_lever).await?;
    helper
        .set_block_default(&wire_pos, &Block::REDSTONE_WIRE)
        .await?;
    helper
        .set_block_default(&wire_far_pos, &Block::REDSTONE_WIRE)
        .await?;

    helper.pull_lever(&lever_pos).await?;
    helper.assert_block_state(&lever_pos, powered_lever).await?;

    let wire_power =
        RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await).power;
    let wire_far_power =
        RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_far_pos).await).power;
    helper.assert_true(
        wire_power == 15 && wire_far_power == 14,
        format!("Expected wire power 15 -> 14 when lever ON, got {wire_power} -> {wire_far_power}"),
    )?;

    helper.pull_lever(&lever_pos).await?;
    helper
        .assert_block_state(&lever_pos, unpowered_lever)
        .await?;
    let wire_power_off =
        RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await).power;
    helper.assert_true(
        wire_power_off == 0,
        format!("Expected wire power 0 when lever OFF, got {wire_power_off}"),
    )?;

    helper.succeed();
    Ok(())
}

pub(super) async fn test_pressure_plate_entity_activation(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;

    let stone_plate_pos = BlockPos::new(1, 1, 1);
    let gold_plate_pos = BlockPos::new(3, 1, 1);

    helper
        .set_block_default(&stone_plate_pos, &Block::STONE_PRESSURE_PLATE)
        .await?;
    helper
        .set_block_default(&gold_plate_pos, &Block::LIGHT_WEIGHTED_PRESSURE_PLATE)
        .await?;

    helper
        .spawn_entity_at("minecraft:armor_stand", Vector3::new(1.5, 1.25, 1.5))
        .await?;
    helper
        .spawn_item_at("minecraft:diamond", 1, Vector3::new(3.5, 1.25, 1.5))
        .await?;

    let powered_stone_plate = {
        let mut props = StonePressurePlateLikeProperties::default(&Block::STONE_PRESSURE_PLATE);
        props.powered = true;
        props.to_state_id(&Block::STONE_PRESSURE_PLATE)
    };
    let powered_gold_plate = {
        let mut props = LightWeightedPressurePlateLikeProperties::default(
            &Block::LIGHT_WEIGHTED_PRESSURE_PLATE,
        );
        props.power = 1;
        props.to_state_id(&Block::LIGHT_WEIGHTED_PRESSURE_PLATE)
    };

    helper.succeed_when(move |helper| async move {
        helper
            .assert_block_state(&stone_plate_pos, powered_stone_plate)
            .await?;
        helper
            .assert_block_state(&gold_plate_pos, powered_gold_plate)
            .await
    })?;
    Ok(())
}
