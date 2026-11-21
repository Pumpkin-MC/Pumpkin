use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use pumpkin_core::server::Server;
use pumpkin_data::dimension::Dimension;
use pumpkin_protocol::java::server::play::Action;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

pub async fn run_player_actions_and_disconnect_tests(
    server: &Arc<Server>,
    mut bot1: HeadlessBot,
    bot2: HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let action_start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player1) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let entity_id = player1.entity_id();

    bot1.set_held_item(4).await?;
    bot1.send_player_command(entity_id, Action::StartSprinting)
        .await?;

    let mut slot_updated = false;
    let mut sprinting_updated = false;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if player1.inventory.get_selected_slot() == 4 {
            slot_updated = true;
        }
        if player1
            .living_entity
            .entity
            .sprinting
            .load(Ordering::Relaxed)
        {
            sprinting_updated = true;
        }
        if slot_updated && sprinting_updated {
            break;
        }
    }

    if !slot_updated {
        return Err(format!(
            "Expected TestBot1 selected hotbar slot 4 after SSetHeldItem, got {}",
            player1.inventory.get_selected_slot()
        ));
    }
    if !sprinting_updated {
        return Err("Expected TestBot1 sprinting=true after SPlayerCommand::StartSprinting".into());
    }

    bot1.send_player_command(entity_id, Action::StopSprinting)
        .await?;
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if !player1
            .living_entity
            .entity
            .sprinting
            .load(Ordering::Relaxed)
        {
            break;
        }
    }

    summary_lines.push(format!(
        "[PASS] bot:held_slot_and_sprinting_actions ({}ms)",
        action_start.elapsed().as_millis()
    ));

    let disconnect_start = Instant::now();
    drop(bot1);
    drop(bot2);
    for _ in 0..MAX_POLL_TICKS {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
        if server.get_player_count() == 0 {
            break;
        }
    }
    let remaining_players = server.get_player_count();
    if remaining_players != 0 {
        return Err(format!(
            "Expected 0 connected players after bot disconnect, found {remaining_players}"
        ));
    }
    summary_lines.push(format!(
        "[PASS] bot:disconnect_cleanup ({}ms)",
        disconnect_start.elapsed().as_millis()
    ));

    Ok(())
}
