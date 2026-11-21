use std::{sync::Arc, time::Instant};

use pumpkin_core::{entity::player::RespawnPoint, server::Server};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;

/// Checks that a disabled dimension cannot be created or used as a respawn destination.
pub async fn run_disabled_dimensions_tests(
    server: &Arc<Server>,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let Some(player) = world.get_player_by_name("TestBot1") else {
        return Err("Could not find 'TestBot1' in Overworld".to_string());
    };
    let world_count = server.worlds.load().len();
    let position = BlockPos::new(1_000_000, world.dimension.min_y, 0);
    let chunk_position = position.chunk_position();
    if world.level.is_chunk_loaded(&chunk_position) {
        return Err("Disabled-dimension respawn test requires an unloaded chunk".to_string());
    }

    for (name, dimension) in [
        ("disabled-nether", Dimension::THE_NETHER),
        ("disabled-end", Dimension::THE_END),
    ] {
        let world_path = server.basic_config.get_world_path().join(name);
        let created = server.create_world(name.to_string(), dimension.clone());
        if !Arc::ptr_eq(&created, &world)
            || server.worlds.load().len() != world_count
            || world_path.exists()
        {
            return Err(format!(
                "Creating disabled dimension {} changed the worlds or filesystem",
                dimension.minecraft_name
            ));
        }

        let previous_respawn = player
            .respawn_point
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(RespawnPoint {
                dimension: dimension.clone(),
                position,
                yaw: 0.0,
                force: true,
            });
        let respawn = player.calculate_respawn_point().await;
        *player
            .respawn_point
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = previous_respawn;

        if respawn.is_some() || world.level.is_chunk_loaded(&chunk_position) {
            return Err(format!(
                "Respawn in disabled dimension {} resolved or loaded Overworld chunks",
                dimension.minecraft_name
            ));
        }
    }

    summary_lines.push(format!(
        "[PASS] bot:disabled_dimensions_create_and_respawn ({}ms)",
        start.elapsed().as_millis()
    ));
    Ok(())
}
