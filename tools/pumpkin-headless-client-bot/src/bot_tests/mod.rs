mod embedded_stop;
mod login_and_movement;
mod sign_authorization;
mod snow_golem;
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

    snow_golem::run_snow_golem_shearing_test(server, summary_lines)?;

    drop(bot1);
    drop(bot2);

    sign_authorization::run_sign_authorization_tests(server, addr, summary_lines).await?;

    embedded_stop::run_embedded_stop_test(summary_lines).await?;

    Ok(())
}
