use pumpkin_data::{
    Block,
    block_properties::{
        DispenserLikeProperties, EastRedstone, Facing, NorthRedstone, RedstoneWireLikeProperties,
        SouthRedstone, TargetProperties, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_target_block_projectile_hit(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let wire_pos = BlockPos::new(2, 1, 1);
    let target_pos = BlockPos::new(2, 1, 2);
    let dispenser_pos = BlockPos::new(2, 1, 3);
    let power_pos = BlockPos::new(1, 1, 3);

    let mut wire_props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    wire_props.north = NorthRedstone::Side;
    wire_props.south = SouthRedstone::Side;
    wire_props.east = EastRedstone::None;
    wire_props.west = WestRedstone::None;
    helper
        .set_block(&wire_pos, wire_props.to_state_id(&Block::REDSTONE_WIRE))
        .await?;
    helper
        .set_block_default(&target_pos, &Block::TARGET)
        .await?;

    let mut dispenser_props = DispenserLikeProperties::default(&Block::DISPENSER);
    dispenser_props.facing = Facing::North;
    helper
        .set_block(
            &dispenser_pos,
            dispenser_props.to_state_id(&Block::DISPENSER),
        )
        .await?;
    helper
        .set_container_item(&dispenser_pos, 0, "minecraft:snowball", 1)
        .await?;

    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            let target_state = helper.block_state_id(&target_pos).await;
            let target_props = TargetProperties::from_state_id(target_state);
            helper.assert_true(
                target_props.power > 0,
                "expected target block power > 0 after snowball impact",
            )?;

            let wire_state = helper.block_state_id(&wire_pos).await;
            let wire_props = RedstoneWireLikeProperties::from_state_id(wire_state);
            helper.assert_true(
                wire_props.power > 0,
                "expected redstone wire adjacent to hit target block to be powered (> 0)",
            )
        })
        .then_idle(10)
        .then_execute(move |helper| async move {
            let target_state = helper.block_state_id(&target_pos).await;
            let target_props = TargetProperties::from_state_id(target_state);
            helper.assert_true(
                target_props.power == 0,
                format!(
                    "expected target block power to reset to 0 after scheduled tick, found {}",
                    target_props.power
                ),
            )?;

            let wire_state = helper.block_state_id(&wire_pos).await;
            let wire_props = RedstoneWireLikeProperties::from_state_id(wire_state);
            helper.assert_true(
                wire_props.power == 0,
                format!(
                    "expected redstone wire to de-energize to 0 after target reset, found {}",
                    wire_props.power
                ),
            )
        })
        .then_succeed();
    Ok(())
}

/// An arrow stuck in a block that is broken falls and sticks in the block below it.
pub(super) async fn test_arrow_falls_onto_block_below(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let support_pos = BlockPos::new(2, 2, 2);
    helper
        .set_block_default(&support_pos, &Block::STONE)
        .await?;
    helper
        .spawn_entity_at("minecraft:arrow", Vector3::new(2.5, 3.2, 2.5))
        .await?;

    helper
        .start_sequence()
        .then_idle(20)
        .then_execute(move |helper| async move {
            helper
                .assert_entity_present_in_box(
                    "minecraft:arrow",
                    BlockPos::new(0, 3, 0),
                    BlockPos::new(5, 4, 5),
                )
                .await?;
            helper.set_air(&support_pos).await
        })
        .then_idle(40)
        .then_execute(|helper| async move {
            helper
                .assert_entity_present_in_box(
                    "minecraft:arrow",
                    BlockPos::new(0, 1, 0),
                    BlockPos::new(5, 2, 5),
                )
                .await
        })
        .then_succeed();
    Ok(())
}
