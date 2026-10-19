mod block_tuning_and_invert;
mod commands_and_gamemode;
mod login_and_movement;
mod player_actions;
mod status_ping;
mod world_interaction;

use std::{net::SocketAddr, sync::Arc};

use pumpkin_core::server::Server;
use pumpkin_gametest::GameTestRunner;

pub async fn run_bot_tests(
    server: &Arc<Server>,
    addr: SocketAddr,
    runner: &mut GameTestRunner,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    status_ping::run_bot_status_ping_test(addr, summary_lines).await?;

    let (mut bot1, bot2) =
        login_and_movement::run_login_and_movement_tests(server, addr, summary_lines).await?;

    commands_and_gamemode::run_commands_and_gamemode_tests(
        server,
        &mut bot1,
        runner,
        summary_lines,
    )
    .await?;

    world_interaction::run_world_interaction_tests(server, &mut bot1, summary_lines).await?;

    block_tuning_and_invert::run_block_tuning_and_invert_tests(server, &mut bot1, summary_lines)
        .await?;

    player_actions::run_player_actions_and_disconnect_tests(server, bot1, bot2, summary_lines)
        .await?;

    Ok(())
}
