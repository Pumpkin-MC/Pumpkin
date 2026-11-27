mod embedded_stop;
mod login_and_movement;
mod peaceful_despawn;
mod status_ping;

use std::{net::SocketAddr, sync::Arc};

use pumpkin_core::server::Server;

pub async fn run_bot_tests(
    server: &Arc<Server>,
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    status_ping::run_bot_status_ping_test(addr, summary_lines).await?;

    let (bot1, bot2) =
        login_and_movement::run_login_and_movement_tests(server, addr, summary_lines).await?;

    // These tests drive the server clock and change server-wide difficulty.
    peaceful_despawn::run_peaceful_despawn_test(server, &bot1, summary_lines)?;
    peaceful_despawn::run_mid_tick_difficulty_test(server, &bot1, summary_lines)?;

    drop(bot1);
    drop(bot2);

    embedded_stop::run_embedded_stop_test(summary_lines).await?;

    Ok(())
}
