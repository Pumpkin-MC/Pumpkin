use std::{
    sync::Arc,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use pumpkin_core::server::{Server, server_test_manager::drain_game_test_queue};
use pumpkin_data::{Block, dimension::Dimension, item::Item};
use pumpkin_gametest::GameTestRunner;
use pumpkin_util::{GameMode, PermissionLvl, math::position::BlockPos};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS, PACKET_TIMEOUT};

pub async fn run_commands_and_gamemode_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    runner: &mut GameTestRunner,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let cmd_start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let dispatcher = server.command_dispatcher.load();
    player1.set_permission_lvl(server, PermissionLvl::Four, &dispatcher);

    // 1. Test `/gamemode creative` and `/give @s minecraft:diamond 5`
    bot1.send_command("gamemode creative").await?;
    bot1.send_command("give @s minecraft:diamond 5").await?;

    let marker_pos = BlockPos::new(
        bot1.position.x.floor() as i32,
        (bot1.position.y.floor() as i32) + 10,
        bot1.position.z.floor() as i32,
    );

    // `/setblock` rejects positions in unloaded chunks, the spawn chunk can briefly be absent.
    let load_start = Instant::now();
    while !world.is_loaded(&marker_pos) {
        if load_start.elapsed() >= PACKET_TIMEOUT {
            return Err(format!(
                "Timed out waiting for the /setblock chunk at {marker_pos:?} to load"
            ));
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
    }

    bot1.send_command(&format!(
        "setblock {} {} {} minecraft:emerald_block",
        marker_pos.0.x, marker_pos.0.y, marker_pos.0.z
    ))
    .await?;

    // 2. Test `/test run minecraft:always_pass`
    bot1.send_command("test run minecraft:always_pass").await?;

    let mut gamemode_updated = false;
    let mut diamond_received = false;
    let mut block_placed = false;
    let mut command_queued_and_ran = false;

    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        drain_game_test_queue(server, runner).await;
        if !runner.is_empty() {
            command_queued_and_ran = true;
        }
        runner.tick().await;

        if player1.gamemode.load() == GameMode::Creative {
            gamemode_updated = true;
        }

        if count_player_diamonds(&player1) == 5 {
            diamond_received = true;
        }

        if world.get_block(&marker_pos).id == Block::EMERALD_BLOCK.id {
            block_placed = true;
        }

        if gamemode_updated
            && diamond_received
            && block_placed
            && command_queued_and_ran
            && runner.is_empty()
        {
            break;
        }
    }

    if !gamemode_updated {
        return Err("Bot `/gamemode creative` did not update player gamemode".to_string());
    }
    if !diamond_received {
        return Err("Bot `/give @s minecraft:diamond 5` did not add 5 diamonds".to_string());
    }
    if !block_placed {
        return Err("Bot `/setblock` did not place emerald_block in the world".to_string());
    }
    if !command_queued_and_ran {
        return Err(
            "Bot 'TestBot1' `/test run minecraft:always_pass` command did not enqueue a GameTest"
                .to_string(),
        );
    }

    summary_lines.push(format!(
        "[PASS] bot:commands_gamemode_give_setblock_test ({}ms)",
        cmd_start.elapsed().as_millis()
    ));

    run_xp_negative_amount_test(server, bot1, &player1, runner, summary_lines).await?;

    Ok(())
}

/// `/xp add` with a negative amount has to clamp at level 0 instead of going
/// negative. The level is read back off the player rather than out of the command
/// reply, so this covers the state and not the message.
async fn run_xp_negative_amount_test(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    player1: &Arc<pumpkin_core::entity::player::Player>,
    runner: &mut GameTestRunner,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let xp_start = Instant::now();
    bot1.send_command("xp set @s 24 levels").await?;

    // Wait for the set to land first, otherwise the clamp check below passes on
    // the starting state before either command has run.
    let mut xp_set = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        runner.tick().await;
        if player1.experience_level.load(Ordering::Relaxed) == 24 {
            xp_set = true;
            break;
        }
    }
    if !xp_set {
        return Err(format!(
            "Bot `/xp set @s 24 levels` left the player on level {}",
            player1.experience_level.load(Ordering::Relaxed)
        ));
    }

    bot1.send_command("xp add @s -100 levels").await?;

    let mut xp_clamped = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        runner.tick().await;
        if player1.experience_level.load(Ordering::Relaxed) == 0
            && player1.experience_points.load(Ordering::Relaxed) == 0
        {
            xp_clamped = true;
            break;
        }
    }
    if !xp_clamped {
        return Err(format!(
            "Bot `/xp add @s -100 levels` left the player on level {} with {} points, expected 0 and 0",
            player1.experience_level.load(Ordering::Relaxed),
            player1.experience_points.load(Ordering::Relaxed)
        ));
    }

    summary_lines.push(format!(
        "[PASS] bot:commands_xp_add_negative_clamps ({}ms)",
        xp_start.elapsed().as_millis()
    ));
    Ok(())
}

fn count_player_diamonds(player: &pumpkin_core::entity::player::Player) -> u8 {
    let inv = player.inventory();
    let main_inv = inv
        .main_inventory
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut count = 0u8;
    for stack in main_inv.iter() {
        if stack.item.id == Item::DIAMOND.id {
            count = count.saturating_add(stack.item_count);
        }
    }
    count
}
