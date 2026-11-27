mod embedded_stop;
mod explosion_knockback;
mod login_and_movement;
mod status_ping;

use std::{net::SocketAddr, sync::Arc};

use pumpkin_core::server::Server;

pub async fn run_bot_tests(
    server: &Arc<Server>,
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    status_ping::run_bot_status_ping_test(addr, summary_lines).await?;

    let (mut bot1, mut bot2) =
        login_and_movement::run_login_and_movement_tests(server, addr, summary_lines).await?;

    explosion_knockback::run_explosion_knockback_test(server, &mut bot1, &mut bot2, summary_lines)
        .await?;

    drop(bot1);
    drop(bot2);

    embedded_stop::run_embedded_stop_test(summary_lines).await?;

    Ok(())
}
