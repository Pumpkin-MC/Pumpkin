use pumpkin_data::{
    Block,
    block_properties::{
        DetectorRailProperties, PoweredRailLikeProperties, RailShapeStraight,
        RedstoneWireLikeProperties,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_powered_rail_propagation(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 10).await?;

    let mut unpowered_props = PoweredRailLikeProperties::default(&Block::POWERED_RAIL);
    unpowered_props.shape = RailShapeStraight::NorthSouth;
    unpowered_props.powered = false;
    let unpowered_rail = unpowered_props.to_state_id(&Block::POWERED_RAIL);

    let mut powered_props = unpowered_props;
    powered_props.powered = true;
    let powered_rail = powered_props.to_state_id(&Block::POWERED_RAIL);

    for z in 1..=8 {
        helper
            .set_block(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    let power_source = BlockPos::new(1, 1, 0);
    helper
        .set_block_default(&power_source, &Block::REDSTONE_BLOCK)
        .await?;

    for z in 1..=8 {
        helper
            .assert_block_state(&BlockPos::new(1, 1, z), powered_rail)
            .await?;
    }

    helper.set_block_default(&power_source, &Block::AIR).await?;

    for z in 1..=8 {
        helper
            .assert_block_state(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    helper.succeed();
    Ok(())
}

pub(super) async fn test_activator_rail_propagation(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 10).await?;

    let mut unpowered_props = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
    unpowered_props.shape = RailShapeStraight::NorthSouth;
    unpowered_props.powered = false;
    let unpowered_rail = unpowered_props.to_state_id(&Block::ACTIVATOR_RAIL);

    let mut powered_props = unpowered_props;
    powered_props.powered = true;
    let powered_rail = powered_props.to_state_id(&Block::ACTIVATOR_RAIL);

    for z in 1..=8 {
        helper
            .set_block(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    let power_source = BlockPos::new(1, 1, 0);
    helper
        .set_block_default(&power_source, &Block::REDSTONE_BLOCK)
        .await?;

    for z in 1..=8 {
        helper
            .assert_block_state(&BlockPos::new(1, 1, z), powered_rail)
            .await?;
    }

    helper.set_block_default(&power_source, &Block::AIR).await?;

    for z in 1..=8 {
        helper
            .assert_block_state(&BlockPos::new(1, 1, z), unpowered_rail)
            .await?;
    }

    helper.succeed();
    Ok(())
}

pub(super) async fn test_detector_rail_minecart(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let rail_pos = BlockPos::new(2, 1, 2);
    let wire_pos = BlockPos::new(3, 1, 2);

    let mut rail_props = DetectorRailProperties::default(&Block::DETECTOR_RAIL);
    rail_props.shape = RailShapeStraight::NorthSouth;
    rail_props.powered = false;
    helper
        .set_block(&rail_pos, rail_props.to_state_id(&Block::DETECTOR_RAIL))
        .await?;
    helper
        .set_block_default(&wire_pos, &Block::REDSTONE_WIRE)
        .await?;

    helper
        .spawn_entity_at("minecraft:minecart", Vector3::new(2.5, 1.1, 2.5))
        .await?;

    helper.succeed_when(move |helper| async move {
        let state = DetectorRailProperties::from_state_id(helper.block_state_id(&rail_pos).await);
        let wire =
            RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await);
        helper.assert_true(
            state.powered && wire.power == 15,
            format!(
                "Expected detector rail powered=true and wire power=15, got powered={} power={}",
                state.powered, wire.power
            ),
        )
    })?;
    Ok(())
}

pub(super) async fn test_minecart_breaks_in_lava(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for (x, z) in [(1, 2), (3, 2), (2, 1), (2, 3)] {
        helper
            .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(2, 1, 2), &Block::LAVA)
        .await?;
    helper
        .spawn_entity_at("minecraft:minecart", Vector3::new(2.5, 1.0, 2.5))
        .await?;

    helper
        .start_sequence()
        .then_execute(
            |helper| async move { helper.assert_entity_present("minecraft:minecart").await },
        )
        .then_wait_until(|helper| async move {
            helper.assert_entity_not_present("minecraft:minecart").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_minecart_breaks_on_cactus(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    helper
        .set_block_default(&BlockPos::new(3, 0, 2), &Block::SAND)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 2), &Block::CACTUS)
        .await?;
    // Pressed against the cactus side, where its collision box (inset by 1/16) stops the minecart.
    helper
        .spawn_entity_at("minecraft:minecart", Vector3::new(2.5725, 1.0, 2.5))
        .await?;

    helper
        .start_sequence()
        .then_execute(
            |helper| async move { helper.assert_entity_present("minecraft:minecart").await },
        )
        .then_wait_until(|helper| async move {
            helper.assert_entity_not_present("minecraft:minecart").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_minecart_breaking_ejects_passenger(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    const RIDING: &str =
        "execute as @e[type=minecraft:armor_stand,distance=..4] on vehicle if entity @s";

    build_stone_floor(&helper, 5, 5).await?;
    helper
        .set_block_default(&BlockPos::new(3, 0, 2), &Block::SAND)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 2), &Block::CACTUS)
        .await?;
    // The rider sits where the minecart is, pressed against the cactus side.
    let cart_pos = Vector3::new(2.5725, 1.0, 2.5);
    helper
        .spawn_entity_at("minecraft:minecart", cart_pos)
        .await?;
    helper
        .spawn_entity_at("minecraft:armor_stand", cart_pos)
        .await?;
    let center = BlockPos::new(2, 1, 2);
    helper
        .execute_command(
            &center,
            "ride @e[type=minecraft:armor_stand,distance=..4,limit=1] mount @e[type=minecraft:minecart,distance=..4,limit=1]",
        )
        .await?;

    helper
        .start_sequence()
        .then_execute(move |helper| async move {
            let riding = helper.execute_command(&center, RIDING).await?;
            helper.assert_true(riding == 1, "expected the armor stand to ride the minecart")
        })
        .then_wait_until(move |helper| async move {
            helper
                .assert_entity_not_present("minecraft:minecart")
                .await?;
            helper
                .assert_entity_present("minecraft:armor_stand")
                .await?;
            let riding = helper.execute_command(&center, RIDING).await.unwrap_or(0);
            helper.assert_true(
                riding == 0,
                "expected the armor stand to stop riding the destroyed minecart",
            )?;
            // A removed vehicle leaves its rider in place instead of a dismount spot next to it.
            helper
                .assert_entity_present_at("minecraft:armor_stand", &center)
                .await
        })
        .then_succeed();
    Ok(())
}
