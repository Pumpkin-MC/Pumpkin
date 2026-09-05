use pumpkin_util::math::vector2::Vector2;
use std::{num::NonZero, sync::Arc};

use pumpkin_protocol::{
    bedrock::client::network_chunk_publisher_update::CNetworkChunkPublisherUpdate,
    java::client::play::CCenterChunk,
};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::{
    entity::{EntityBase, player::Player},
    net::ClientPlatform,
};

pub fn get_view_distance(player: &Player) -> NonZero<u8> {
    let fallback = NonZero::new(2).unwrap_or(NonZero::<u8>::MIN);
    let Some(server) = player.world().server.upgrade() else {
        return fallback;
    };
    let max_view_distance = match player.client.as_ref() {
        ClientPlatform::Java(_) => server.advanced_config.networking.java.view_distance,
        ClientPlatform::Bedrock(_) => server.advanced_config.networking.bedrock.view_distance,
    };
    player
        .config
        .load()
        .view_distance
        .clamp(fallback, max_view_distance)
}

// Checks if the target chunk is within the view distance
// of the center chunk. Uses Chebyshev distance.
#[must_use]
#[inline]
pub fn is_within_view_distance(
    center: Vector2<i32>,
    target: Vector2<i32>,
    view_distance: i32,
) -> bool {
    (target.x - center.x).abs().max((target.y - center.y).abs()) <= view_distance
}

#[allow(clippy::too_many_lines)]
pub fn update_position(player: &Arc<Player>) {
    let entity = &player.get_entity();
    let new_chunk_center = entity.chunk_pos.load();
    let old_cylindrical = player.watched_section.load();

    // This does break when a new player spawns
    // if old_cylindrical.center == new_chunk_center {
    //     return;
    // }

    let view_distance = get_view_distance(player);
    let new_cylindrical = Cylindrical::new(new_chunk_center, view_distance);

    if old_cylindrical == new_cylindrical {
        return;
    }

    match player.client.as_ref() {
        ClientPlatform::Java(java_client) => {
            java_client.try_send_packet(&CCenterChunk {
                chunk_x: new_chunk_center.x.into(),
                chunk_z: new_chunk_center.y.into(),
            });
        }
        ClientPlatform::Bedrock(bedrock_client) => {
            if let Ok(data) = bedrock_client.serialize_packet(&CNetworkChunkPublisherUpdate::new(
                player.get_entity().block_pos.load(),
                u32::from(view_distance.get()) * 16,
            )) {
                bedrock_client.try_enqueue_packet(data);
            }
        }
    }
    let (loading_iter, unloading_iter) =
        Cylindrical::changed_chunks(old_cylindrical, new_cylindrical);
    let loading_chunks: Vec<_> = loading_iter.collect();
    let unloading_chunks: Vec<_> = unloading_iter.collect();

    let world = player.world();
    let level = &world.level;
    let mut held_tickets = player
        .held_chunk_tickets
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let is_spectator = player.is_spectator();
    let spectators_generate_chunks = world
        .level_info
        .load()
        .game_rules
        .spectators_generate_chunks;

    let new_view_level = (!is_spectator || spectators_generate_chunks).then(|| {
        pumpkin_world::chunk_system::ChunkLoading::get_level_from_view_distance(
            u8::from(view_distance) + 1,
        )
    });

    let new_sim_level = (!is_spectator).then(|| {
        let sim_dist = world.server.upgrade().map_or(10, |s| {
            s.advanced_config.networking.java.simulation_distance.get()
        });
        pumpkin_world::chunk_system::ChunkLoading::get_level_from_simulation_distance(sim_dist)
    });

    {
        let mut lock = level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(view) = new_view_level {
            lock.add_ticket(new_chunk_center, view);
        }
        if let Some(sim) = new_sim_level {
            lock.add_ticket(new_chunk_center, sim);
        }

        if let Some((held_view, held_sim)) = held_tickets.replace((new_view_level, new_sim_level)) {
            if let Some(view) = held_view {
                lock.remove_ticket(old_cylindrical.center, view);
            }
            if let Some(sim) = held_sim {
                lock.remove_ticket(old_cylindrical.center, sim);
            }
        }
        lock.send_change();
    };
    drop(held_tickets);

    {
        let mut sender = player
            .chunk_sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for pos in &unloading_chunks {
            sender.unload_chunk(&player.client, *pos);
        }
        for pos in &loading_chunks {
            sender.enqueue_chunk(*pos);
        }
    }
    player.watched_section.store(new_cylindrical);

    // Make sure the watched section and the chunk watcher updates are async atomic. We want to
    // ensure what we unload when the player disconnects is correct.
    if !loading_chunks.is_empty() || !unloading_chunks.is_empty() {
        let level = world.level.clone();
        let world_clone = world.clone();
        let loading_chunks_clone = loading_chunks.clone();
        let unloading_chunks_clone = unloading_chunks;

        if let Some(server) = world.server.upgrade() {
            server.spawn_task(async move {
                level
                    .mark_chunks_as_newly_watched(&loading_chunks_clone)
                    .await;
                let chunks_to_clean = level
                    .mark_chunks_as_not_watched(&unloading_chunks_clone)
                    .await;

                if !chunks_to_clean.is_empty() {
                    world_clone
                        .remove_entities_in_chunks(&chunks_to_clean)
                        .await;
                    world_clone.level.clean_entity_chunks(&chunks_to_clean);
                }
            });
        }
    }

    if !loading_chunks.is_empty() {
        world.spawn_world_entity_chunks(player.clone(), loading_chunks, new_chunk_center);
    }
    world.entity_tracker.update_player_position(player, &world);
}
