use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::server::Server;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::vector3::Vector3;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_login_and_movement_tests(
    server: &Arc<Server>,
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(HeadlessBot, HeadlessBot), String> {
    let join_start = Instant::now();
    let (bot1_res, bot2_res) = tokio::join!(
        HeadlessBot::connect_and_join(addr, "TestBot1"),
        HeadlessBot::connect_and_join(addr, "TestBot2"),
    );
    let mut bot1 = bot1_res?;
    let bot2 = bot2_res?;

    let mut bots_joined = false;
    for _ in 0..MAX_POLL_TICKS {
        server.tick();
        tokio::time::sleep(Duration::from_millis(5)).await;
        if server.get_player_count() == 2 {
            bots_joined = true;
            break;
        }
    }

    if !bots_joined {
        let player_count = server.get_player_count();
        return Err(format!(
            "Expected 2 connected bots on server, found {player_count}"
        ));
    }
    summary_lines.push(format!(
        "[PASS] bot:multi_player_login_and_spawn (2 bots joined in {}ms)",
        join_start.elapsed().as_millis()
    ));

    let move_start = Instant::now();
    let target_pos = Vector3::new(
        bot1.position.x + 1.5,
        bot1.position.y,
        bot1.position.z + 0.5,
    );
    bot1.move_to(target_pos, true).await?;

    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let mut bot_moved = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if let Some(player1) = world.get_player_by_name("TestBot1") {
            let actual_pos = player1.living_entity.entity.pos.load();
            if (actual_pos.x - target_pos.x).abs() < 0.01
                && (actual_pos.z - target_pos.z).abs() < 0.01
            {
                bot_moved = true;
                break;
            }
        }
    }

    if !bot_moved {
        return Err("Bot 'TestBot1' position update was not applied on the server".to_string());
    }
    summary_lines.push(format!(
        "[PASS] bot:movement_sync ({}ms)",
        move_start.elapsed().as_millis()
    ));

    Ok((bot1, bot2))
}
