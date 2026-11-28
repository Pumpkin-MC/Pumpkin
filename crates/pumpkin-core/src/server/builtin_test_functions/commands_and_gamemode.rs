use pumpkin_data::{Block, item::Item};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};

use crate::{command::CommandSender, server::server_test_manager::server_world};

pub(super) async fn test_ban_ip_rejects_simulated_players(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 3, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "BanIpTester",
            Vector3::new(1.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    player.set_permission_level(PermissionLvl::Three)?;

    let server = server_world(&test)?
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let ban_count = || {
        server
            .data
            .banned_ip_list
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .banned_ips
            .len()
    };
    let before = ban_count();
    let result = player.run_command(&format!("ban-ip {}", player.name()));
    test.assert_true(
        matches!(result, Err(GameTestError::World(message)) if message.contains("no IP address")),
        "/ban-ip did not report that the simulated player has no IP address",
    )?;
    test.assert_true(ban_count() == before, "/ban-ip changed the ban list")?;
    test.succeed();
    Ok(())
}

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

pub(super) async fn test_list_counts_visible_simulated_players(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(3, 3, 3).await?;
    test.build_floor(3, 3, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "ListTester",
            Vector3::new(1.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    let server = server_world(&test)?
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let source = CommandSender::Dummy.into_source(&server);
    let dispatcher = server.command_dispatcher.load();
    let list_count = || {
        dispatcher
            .execute_input("list", &source)
            .map_err(|error| GameTestError::World(error.message.get_text()))
    };

    player.set_visible_in_list(false);
    let hidden_count = list_count()?;
    player.set_visible_in_list(true);
    let visible_count = list_count();
    player.set_visible_in_list(false);
    test.assert_true(
        visible_count? == hidden_count + 1,
        "/list did not count a simulated player marked visible",
    )?;
    test.succeed();
    Ok(())
}
