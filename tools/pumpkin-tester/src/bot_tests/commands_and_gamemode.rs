use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_core::server::Server;
use pumpkin_data::{Block, dimension::Dimension, item::Item};
use pumpkin_util::{GameMode, PermissionLvl, math::position::BlockPos};

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_commands_and_gamemode_tests(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let cmd_start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let dispatcher = server.command_dispatcher.load();
    player1.set_permission_lvl(server, PermissionLvl::Four, &dispatcher);

    bot1.send_command("gamemode creative").await?;
    bot1.send_command("give @s minecraft:diamond 5").await?;

    let marker_pos = BlockPos::new(
        bot1.position.x.floor() as i32,
        (bot1.position.y.floor() as i32) + 10,
        bot1.position.z.floor() as i32,
    );
    let _ = world.get_block(&marker_pos);
    bot1.send_command(&format!(
        "setblock {} {} {} minecraft:emerald_block",
        marker_pos.0.x, marker_pos.0.y, marker_pos.0.z
    ))
    .await?;

    let mut gamemode_updated = false;
    let mut diamond_received = false;
    let mut block_placed = false;

    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();

        if player1.gamemode.load() == GameMode::Creative {
            gamemode_updated = true;
        }

        if count_player_diamonds(&player1) == 5 {
            diamond_received = true;
        }

        if world.get_block(&marker_pos).id == Block::EMERALD_BLOCK.id {
            block_placed = true;
        }

        if gamemode_updated && diamond_received && block_placed {
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
    summary_lines.push(format!(
        "[PASS] bot:commands_gamemode_give_setblock ({}ms)",
        cmd_start.elapsed().as_millis()
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
