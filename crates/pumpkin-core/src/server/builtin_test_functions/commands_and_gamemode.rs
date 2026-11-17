use pumpkin_data::{Block, item::Item};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};

pub(super) async fn test_commands_gamemode_give_setblock(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "CommandTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;
    player.set_permission_level(PermissionLvl::Four)?;

    for command in ["gamemode creative", "give @s minecraft:diamond 5"] {
        player.run_command(command)?;
    }
    let marker = BlockPos::new(3, 2, 3);
    let world_marker = test.absolute_pos(&marker);
    player.run_command(&format!(
        "setblock {} {} {} minecraft:emerald_block",
        world_marker.0.x, world_marker.0.y, world_marker.0.z
    ))?;

    test.assert_true(
        player.gamemode() == GameMode::Creative,
        "Player /gamemode did not change the simulated player's mode",
    )?;
    let diamond_count = player.inventory_count(&Item::DIAMOND);
    test.assert_true(
        diamond_count == 5,
        "Player /give @s did not add five diamonds",
    )?;
    test.assert_true(
        test.block_state_id(&marker).await == Block::EMERALD_BLOCK.default_state.id,
        "Player /setblock did not place the emerald block",
    )?;
    test.succeed();
    Ok(())
}
