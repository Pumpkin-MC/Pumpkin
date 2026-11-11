use pumpkin_data::{Block, item::Item};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};

use crate::server::server_test_manager::ServerSimulatedPlayer;

use super::build_stone_floor;

pub(super) async fn test_commands_gamemode_give_setblock(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let simulated = test
        .spawn_simulated_player(
            "CommandTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();
    let world = player.world();
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let dispatcher = server.command_dispatcher.load();
    player.set_permission_lvl(&server, PermissionLvl::Four, &dispatcher);
    let source = player.get_command_source(&server).with_silent();

    for command in ["gamemode creative", "give @s minecraft:diamond 5"] {
        dispatcher
            .execute_input(command, &source)
            .map_err(|error| GameTestError::World(error.message.get_text()))?;
    }
    let marker = BlockPos::new(3, 2, 3);
    let world_marker = test.absolute_pos(&marker);
    dispatcher
        .execute_input(
            &format!(
                "setblock {} {} {} minecraft:emerald_block",
                world_marker.0.x, world_marker.0.y, world_marker.0.z
            ),
            &source,
        )
        .map_err(|error| GameTestError::World(error.message.get_text()))?;

    test.assert_true(
        player.gamemode.load() == GameMode::Creative,
        "Player /gamemode did not change the simulated player's mode",
    )?;
    let diamond_count: u32 = player
        .inventory()
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .filter(|stack| stack.item.id == Item::DIAMOND.id)
        .map(|stack| u32::from(stack.item_count))
        .sum();
    test.assert_true(
        diamond_count == 5,
        "Player /give @s did not add five diamonds",
    )?;
    test.assert_true(
        world.get_block(&world_marker).id == Block::EMERALD_BLOCK.id,
        "Player /setblock did not place the emerald block",
    )?;
    test.succeed();
    Ok(())
}
