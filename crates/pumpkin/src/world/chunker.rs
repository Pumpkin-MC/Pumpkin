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

    // `change_world_chunks` then `set_world` run with no await between them, so `player.world()`
    // is already the destination when this runs after a portal teleport.
    let world = player.world();
    // Before replacing this player's tickets on the unloading chunks below: that can let
    // `GenerationSchedule` evict their raw block data before the tick loop persists any live
    // BE in them. See `World::flush_block_entities`.
    world.flush_block_entities(&unloading_chunks);
    player.replace_chunk_tickets(
        &world.level,
        old_cylindrical.center,
        new_chunk_center,
        view_distance.into(),
    );
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

    // Drop the entities of the leaving chunks on this client before unwatching them: past that
    // point the player is no longer a chunk watcher, so no chunk-scoped broadcast can reach them.
    // Covers every unloading chunk, not just those with zero watchers. Visibility is per player
    // (`ChunkMap.TrackedEntity::removePairing`), so a chunk another player still watches must
    // equally stop being rendered here.
    world.despawn_entities_in_chunks_for_player(player, &unloading_chunks);

    // Watcher IO is async. Tickets must land before `spawn_world_entity_chunks`.
    if !loading_chunks.is_empty() || !unloading_chunks.is_empty() {
        let level = world.level.clone();
        let world_clone = world.clone();
        let player = player.clone();
        if let Some(server) = world.server.upgrade() {
            server.spawn_task(async move {
                if !loading_chunks.is_empty() {
                    level.mark_chunks_as_newly_watched(&loading_chunks).await;
                }
                if !unloading_chunks.is_empty() {
                    let chunks_to_clean = level.mark_chunks_as_not_watched(&unloading_chunks).await;
                    world_clone.queue_chunk_removal(&chunks_to_clean);
                }
                if !loading_chunks.is_empty() {
                    world_clone.spawn_world_entity_chunks(player, loading_chunks, new_chunk_center);
                }
            });
        } else if !loading_chunks.is_empty() {
            world.spawn_world_entity_chunks(player, loading_chunks, new_chunk_center);
        }
    }
}
