use serde_json::{Value, json};
use std::{
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use super::Server;

pub(crate) fn counters(server: &Server) -> Value {
    pumpkin_profiler::without_tracking(|| collect_counters(server))
}

fn collect_counters(server: &Server) -> Value {
    let worlds: Vec<_> = server
        .worlds
        .load()
        .iter()
        .map(|world| {
            json!({
                "id": world.uuid.to_string(), "dimension": world.dimension.minecraft_name,
                "players": world.players.load().len(), "entities": world.entities.load().len(),
                "block_entity_chunks": world.block_entities.len(),
                "level": world.level.memory_counters()
            })
        })
        .collect();
    json!({
        "captured_unix_millis": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
        "tick": server.tick_count.load(Ordering::Relaxed), "mspt": server.get_mspt(),
        "tracked_server_tasks": server.tasks.len(),
        "target_tps": server.tick_rate_manager.tickrate(),
        "java_view_distance": server.advanced_config.networking.java.view_distance.get(),
        "java_simulation_distance": server.advanced_config.networking.java.simulation_distance.get(),
        "worlds": worlds
    })
}
