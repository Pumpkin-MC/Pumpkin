use pumpkin_data::{
    Block,
    block_properties::{CommandBlockLikeProperties, Facing, FacingHopper, HopperLikeProperties},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_falling_sand_lands(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;

    let spawn_pos = BlockPos::new(1, 4, 1);
    let land_pos = BlockPos::new(1, 1, 1);
    helper.set_block_default(&spawn_pos, &Block::SAND).await?;

    helper.succeed_when(move |helper| async move {
        helper.assert_block_present(&spawn_pos, &Block::AIR).await?;
        helper.assert_block_present(&land_pos, &Block::SAND).await
    })?;
    Ok(())
}

pub(super) async fn test_command_block_chain_execution(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;

    let power_pos = BlockPos::new(1, 1, 1);
    let impulse_pos = BlockPos::new(2, 1, 1);
    let chain_pos = BlockPos::new(3, 1, 1);
    let impulse_marker = BlockPos::new(2, 2, 1);
    let chain_marker = BlockPos::new(3, 2, 1);

    let mut impulse_props = CommandBlockLikeProperties::default(&Block::COMMAND_BLOCK);
    impulse_props.facing = Facing::East;
    impulse_props.conditional = false;
    let mut impulse_nbt = NbtCompound::new();
    impulse_nbt.put_string("id", "minecraft:command_block".to_string());
    impulse_nbt.put_string(
        "Command",
        "setblock ~ ~1 ~ minecraft:gold_block".to_string(),
    );
    impulse_nbt.put_bool("auto", false);
    helper
        .set_block_with_nbt(
            &impulse_pos,
            impulse_props.to_state_id(&Block::COMMAND_BLOCK),
            &impulse_nbt,
        )
        .await?;

    let mut chain_props = CommandBlockLikeProperties::default(&Block::CHAIN_COMMAND_BLOCK);
    chain_props.facing = Facing::East;
    chain_props.conditional = true;
    let mut chain_nbt = NbtCompound::new();
    chain_nbt.put_string("id", "minecraft:command_block".to_string());
    chain_nbt.put_string(
        "Command",
        "setblock ~ ~1 ~ minecraft:diamond_block".to_string(),
    );
    chain_nbt.put_bool("auto", true);
    helper
        .set_block_with_nbt(
            &chain_pos,
            chain_props.to_state_id(&Block::CHAIN_COMMAND_BLOCK),
            &chain_nbt,
        )
        .await?;

    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            helper
                .assert_block_present(&impulse_marker, &Block::GOLD_BLOCK)
                .await?;
            helper
                .assert_block_present(&chain_marker, &Block::DIAMOND_BLOCK)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_transfers_to_chest(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;

    let bottom_chest = BlockPos::new(1, 1, 1);
    let hopper_pos = BlockPos::new(1, 2, 1);
    let top_chest = BlockPos::new(1, 3, 1);

    helper
        .set_block_default(&bottom_chest, &Block::CHEST)
        .await?;

    let mut hopper_props = HopperLikeProperties::default(&Block::HOPPER);
    hopper_props.facing = FacingHopper::Down;
    hopper_props.enabled = true;
    helper
        .set_block(&hopper_pos, hopper_props.to_state_id(&Block::HOPPER))
        .await?;

    helper.set_block_default(&top_chest, &Block::CHEST).await?;
    helper
        .set_container_item(&top_chest, 0, "minecraft:iron_ingot", 2)
        .await?;

    helper.succeed_when(move |helper| async move {
        helper.assert_container_empty(&top_chest).await?;
        helper.assert_container_empty(&hopper_pos).await?;
        helper
            .assert_container_contains_count(&bottom_chest, "minecraft:iron_ingot", 2)
            .await
    })?;
    Ok(())
}

pub(super) async fn test_tnt_redstone_ignites(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let tnt_pos = BlockPos::new(2, 1, 2);
    let power_pos = BlockPos::new(2, 1, 1);

    helper.set_block_default(&tnt_pos, &Block::TNT).await?;
    helper
        .set_block_default(&power_pos, &Block::REDSTONE_BLOCK)
        .await?;

    helper.assert_block_present(&tnt_pos, &Block::AIR).await?;
    helper.assert_entity_present("minecraft:tnt").await?;

    helper.succeed();
    Ok(())
}
