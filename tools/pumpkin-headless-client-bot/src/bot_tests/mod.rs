mod embedded_stop;
mod login_and_movement;
mod peaceful_despawn;
mod status_ping;

use std::{net::SocketAddr, sync::Arc};

use pumpkin_core::{entity::EntityBase, server::Server};

pub async fn run_bot_tests(
    server: &Arc<Server>,
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    status_ping::run_bot_status_ping_test(addr, summary_lines).await?;

    let (bot1, bot2) =
        login_and_movement::run_login_and_movement_tests(server, addr, summary_lines).await?;

    // Use this isolated server to control the clock and difficulty without affecting
    // other GameTests sharing a world.
    let player = server
        .get_player_by_name(&bot1.name)
        .ok_or("Missing despawn test player")?;
    let chunk = player.get_entity().chunk_pos.load();
    // The bot's movement can finish before the fixture chunk loads, and mobs only tick in loaded chunks.
    player.world().level.get_or_fetch_chunk(chunk, |_| ()).await;
    peaceful_despawn::run_peaceful_despawn_test(server, &bot1, summary_lines)?;
    peaceful_despawn::run_mid_tick_difficulty_test(server, &bot1, summary_lines)?;
    peaceful_despawn::run_late_peaceful_wither_test(server, &bot1, summary_lines)?;

    drop(bot1);
    drop(bot2);

    embedded_stop::run_embedded_stop_test(summary_lines).await?;

    Ok(())
}
