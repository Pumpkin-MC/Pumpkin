use pumpkin_data::{
    Block,
    block_properties::{
        DetectorRailProperties, PoweredRailLikeProperties, RailShapeStraight,
        RedstoneWireLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

pub(super) async fn test_powered_rail_propagation(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 10).await?;
    test.build_floor(3, 10, &Block::STONE).await?;

    let mut unpowered_props = PoweredRailLikeProperties::default(&Block::POWERED_RAIL);
    unpowered_props.shape = RailShapeStraight::NorthSouth;
    unpowered_props.powered = false;
    let unpowered_rail = unpowered_props.to_state_id(&Block::POWERED_RAIL);

    let mut powered_props = unpowered_props;
    powered_props.powered = true;
    let powered_rail = powered_props.to_state_id(&Block::POWERED_RAIL);

    for z in 1..=8 {
        test.set_block(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    let power_source = BlockPos::new(1, 1, 0);
    test.set_block_default(&power_source, &Block::REDSTONE_BLOCK)
        .await?;

    for z in 1..=8 {
        test.assert_block_state(&BlockPos::new(1, 1, z), powered_rail)
            .await?;
    }

    test.set_block_default(&power_source, &Block::AIR).await?;

    for z in 1..=8 {
        test.assert_block_state(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    test.succeed();
    Ok(())
}

pub(super) async fn test_activator_rail_propagation(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 10).await?;
    test.build_floor(3, 10, &Block::STONE).await?;

    let mut unpowered_props = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
    unpowered_props.shape = RailShapeStraight::NorthSouth;
    unpowered_props.powered = false;
    let unpowered_rail = unpowered_props.to_state_id(&Block::ACTIVATOR_RAIL);

    let mut powered_props = unpowered_props;
    powered_props.powered = true;
    let powered_rail = powered_props.to_state_id(&Block::ACTIVATOR_RAIL);

    for z in 1..=8 {
        test.set_block(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    let power_source = BlockPos::new(1, 1, 0);
    test.set_block_default(&power_source, &Block::REDSTONE_BLOCK)
        .await?;

    for z in 1..=8 {
        test.assert_block_state(&BlockPos::new(1, 1, z), powered_rail)
            .await?;
    }

    test.set_block_default(&power_source, &Block::AIR).await?;

    for z in 1..=8 {
        test.assert_block_state(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    test.succeed();
    Ok(())
}

pub(super) async fn test_detector_rail_minecart(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let rail_pos = BlockPos::new(2, 1, 2);
    let wire_pos = BlockPos::new(3, 1, 2);

    let mut rail_props = DetectorRailProperties::default(&Block::DETECTOR_RAIL);
    rail_props.shape = RailShapeStraight::NorthSouth;
    rail_props.powered = false;
    test.set_block(&rail_pos, rail_props.to_state_id(&Block::DETECTOR_RAIL))
        .await?;
    test.set_block_default(&wire_pos, &Block::REDSTONE_WIRE)
        .await?;

    test.spawn_entity("minecraft:minecart", Vector3::new(2.5, 1.1, 2.5))
        .await?;

    test.succeed_when(move |test| async move {
        let state = DetectorRailProperties::from_state_id(test.block_state_id(&rail_pos).await);
        let wire = RedstoneWireLikeProperties::from_state_id(test.block_state_id(&wire_pos).await);
        test.assert_true(
            state.powered && wire.power == 15,
            format!(
                "Expected detector rail powered=true and wire power=15, got powered={} power={}",
                state.powered, wire.power
            ),
        )
    })?;
    Ok(())
}

/// Vanilla `MinecartTNT.causeFallDamage`: a fall of 3 blocks or more sets the cart off.
pub(super) async fn test_tnt_minecart_explodes_after_fall(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;
    helper
        .spawn_entity_at("minecraft:tnt_minecart", Vector3::new(1.5, 5.0, 1.5))
        .await?;

    helper
        .start_sequence()
        .then_execute(|helper| async move {
            helper.assert_entity_present("minecraft:tnt_minecart").await
        })
        .then_wait_until(|helper| async move {
            helper
                .assert_entity_not_present("minecraft:tnt_minecart")
                .await
        })
        .then_succeed();
    Ok(())
}

/// Vanilla `AbstractMinecart.getDismountLocationForPassenger`: a cart at yaw 0 drops its
/// passenger north of it with the default movement, which rides sideways, and west of it with
/// the experimental one.
pub(super) async fn test_minecart_dismounts_beside_its_motion(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let cart = BlockPos::new(2, 1, 2);
    helper
        .execute_command(&cart, "summon minecraft:minecart ~ ~ ~")
        .await?;
    helper
        .execute_command(&cart, "summon minecraft:zombie ~ ~ ~ {NoAI:1b}")
        .await?;
    helper
        .execute_command(
            &cart,
            "ride @e[type=minecraft:zombie,limit=1,sort=nearest] mount @e[type=minecraft:minecart,limit=1,sort=nearest]",
        )
        .await?;

    helper
        .start_sequence()
        .then_idle(1)
        .then_execute(move |helper| async move {
            helper
                .execute_command(
                    &cart,
                    "ride @e[type=minecraft:zombie,limit=1,sort=nearest] dismount",
                )
                .await?;
            Ok(())
        })
        .then_execute(move |helper| async move {
            // A second dismount fails once the zombie no longer rides.
            let still_riding = helper
                .execute_command(
                    &cart,
                    "ride @e[type=minecraft:zombie,limit=1,sort=nearest] dismount",
                )
                .await
                .is_ok();
            helper.assert_true(!still_riding, "the zombie still rides the cart")?;
            let (expected, other) = if helper
                .is_feature_enabled("minecraft:minecart_improvements")
                .await?
            {
                (BlockPos::new(1, 1, 2), BlockPos::new(2, 1, 1))
            } else {
                (BlockPos::new(2, 1, 1), BlockPos::new(1, 1, 2))
            };
            helper
                .assert_entity_present_at("minecraft:zombie", &expected)
                .await?;
            helper
                .assert_entity_not_present_at("minecraft:zombie", &other)
                .await
        })
        .then_succeed();
    Ok(())
}
