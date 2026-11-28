use pumpkin_data::{
    Block,
    block_properties::{
        DetectorRailProperties, FacingHopper, HopperLikeProperties, PoweredRailLikeProperties,
        RailLikeProperties, RailShape, RailShapeStraight, RedstoneWireLikeProperties,
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

/// The world direction of the test's +z axis. Entity motion and rotation from commands, and
/// block states from `set_block`, don't turn with the test.
fn world_south(test: &GameTestContext) -> (i32, i32) {
    let from = test.absolute_pos(&BlockPos::new(0, 0, 0));
    let to = test.absolute_pos(&BlockPos::new(0, 0, 1));
    (to.0.x - from.0.x, to.0.z - from.0.z)
}

/// Vanilla `MinecartTNT.causeFallDamage`: a fall of 3 blocks or more sets the cart off.
pub(super) async fn test_tnt_minecart_explodes_after_fall(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 6, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;
    test.spawn_entity("minecraft:tnt_minecart", Vector3::new(1.5, 5.0, 1.5))
        .await?;

    test.start_sequence()
        .then_execute(
            |test| async move { test.assert_entity_present("minecraft:tnt_minecart").await },
        )
        .then_wait_until(|test| async move {
            test.assert_entity_not_present("minecraft:tnt_minecart")
                .await
        })
        .then_succeed();
    Ok(())
}

/// Vanilla `AbstractMinecart.getDismountLocationForPassenger`: a cart facing the test's south
/// drops its passenger north of it with the default movement, which rides sideways, and west of
/// it with the experimental one.
pub(super) async fn test_minecart_dismounts_beside_its_motion(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let cart = BlockPos::new(2, 1, 2);
    let (dx, dz) = world_south(&test);
    let yaw = f64::from(-dx).atan2(f64::from(dz)).to_degrees();
    test.execute_command(
        &cart,
        &format!("summon minecraft:minecart ~ ~ ~ {{Rotation:[{yaw:.1}f,0.0f]}}"),
    )
    .await?;
    test.execute_command(&cart, "summon minecraft:zombie ~ ~ ~ {NoAI:1b}")
        .await?;
    test.execute_command(
        &cart,
        "ride @e[type=minecraft:zombie,limit=1,sort=nearest] mount @e[type=minecraft:minecart,limit=1,sort=nearest]",
    )
    .await?;

    test.start_sequence()
        .then_idle(1)
        .then_execute(move |test| async move {
            test.execute_command(
                &cart,
                "ride @e[type=minecraft:zombie,limit=1,sort=nearest] dismount",
            )
            .await?;
            Ok(())
        })
        .then_execute(move |test| async move {
            // A second dismount fails once the zombie no longer rides.
            let still_riding = test
                .execute_command(
                    &cart,
                    "ride @e[type=minecraft:zombie,limit=1,sort=nearest] dismount",
                )
                .await
                .is_ok();
            test.assert_true(!still_riding, "the zombie still rides the cart")?;
            let (expected, other) = if test
                .is_feature_enabled("minecraft:minecart_improvements")
                .await?
            {
                (BlockPos::new(1, 1, 2), BlockPos::new(2, 1, 1))
            } else {
                (BlockPos::new(2, 1, 1), BlockPos::new(1, 1, 2))
            };
            test.assert_entity_present_at("minecraft:zombie", &expected)
                .await?;
            test.assert_entity_not_present_at("minecraft:zombie", &other)
                .await
        })
        .then_succeed();
    Ok(())
}

/// Vanilla `pushAndPickupEntities`: a moving cart picks up a boat (MC-113871). The hopper at
/// the end of the track can only reach the chest boat's items if the boat rode there.
pub(super) async fn test_minecart_carries_chest_boat_to_hopper(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 4, 10).await?;
    test.build_floor(3, 10, &Block::STONE).await?;

    let hopper_pos = BlockPos::new(1, 0, 8);
    let mut hopper_props = HopperLikeProperties::default(&Block::HOPPER);
    hopper_props.facing = FacingHopper::East;
    hopper_props.enabled = true;
    test.set_block(&hopper_pos, hopper_props.to_state_id(&Block::HOPPER))
        .await?;
    let (dx, dz) = world_south(&test);
    let mut rail_props = RailLikeProperties::default(&Block::RAIL);
    rail_props.shape = if dx == 0 {
        RailShape::NorthSouth
    } else {
        RailShape::EastWest
    };
    let rail = rail_props.to_state_id(&Block::RAIL);
    for z in 1..=8 {
        test.set_block(&BlockPos::new(1, 1, z), rail).await?;
    }
    // Stops the cart with the boat above the hopper.
    test.set_block_default(&BlockPos::new(1, 1, 9), &Block::STONE)
        .await?;

    let boat = BlockPos::new(1, 1, 3);
    test.execute_command(
        &boat,
        "summon minecraft:oak_chest_boat ~ ~ ~ {Items:[{Slot:0b,id:\"minecraft:diamond\",count:3}]}",
    )
    .await?;
    // At most 0.2 blocks per tick, the cart ends a tick within the 0.2 pickup margin before it
    // would bump into the boat and stop.
    test.execute_command(
        &BlockPos::new(1, 1, 1),
        &format!(
            "summon minecraft:minecart ~ ~ ~ {{Motion:[{:.1}d,0.0d,{:.1}d]}}",
            f64::from(dx) * 0.2,
            f64::from(dz) * 0.2
        ),
    )
    .await?;

    test.succeed_when(move |test| async move {
        test.assert_container_contains(&hopper_pos, "minecraft:diamond")
            .await
    })?;
    Ok(())
}
