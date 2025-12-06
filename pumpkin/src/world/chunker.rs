use std::{num::NonZeroU8, sync::Arc};

use pumpkin_protocol::java::client::play::{CCenterChunk, CUnloadChunk};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::{entity::player::Player, net::ClientPlatform};

pub async fn get_view_distance(player: &Player) -> NonZeroU8 {
    let server = player.world().server.upgrade().unwrap();

    player.config.read().await.view_distance.clamp(
        NonZeroU8::new(2).unwrap(),
        server.basic_config.view_distance,
    )
}

pub async fn update_position(player: &Arc<Player>) {
    let entity = &player.living_entity.entity;

    let view_distance = get_view_distance(player).await;
    let new_chunk_center = entity.chunk_pos.load();

    let old_cylindrical = player.watched_section.load();
    let new_cylindrical = Cylindrical::new(new_chunk_center, view_distance);

    if old_cylindrical != new_cylindrical {
        if let ClientPlatform::Java(java) = &player.client {
            java.send_packet_now(&CCenterChunk {
                chunk_x: new_chunk_center.x.into(),
                chunk_z: new_chunk_center.y.into(),
            })
            .await;
        }
        let mut loading_chunks = Vec::new();
        let mut unloading_chunks = Vec::new();
        Cylindrical::for_each_changed_chunk(
            old_cylindrical,
            new_cylindrical,
            &mut loading_chunks,
            &mut unloading_chunks,
        );

        // Make sure the watched section and the chunk watcher updates are async atomic. We want to
        // ensure what we unload when the player disconnects is correct.
        let level = &entity.world.level;
        level.mark_chunks_as_newly_watched(&loading_chunks).await;
        let chunks_to_clean = level.mark_chunks_as_not_watched(&unloading_chunks).await;

        {
            let mut chunk_manager = player.chunk_manager.lock().await;
            chunk_manager.update_center_and_view_distance(
                new_chunk_center,
                view_distance.into(),
                level,
            );
        };

        player.watched_section.store(new_cylindrical);

        if !chunks_to_clean.is_empty() {
            // level.clean_chunks(&chunks_to_clean).await;
            for chunk in unloading_chunks {
                player
                    .client
                    .enqueue_packet(&CUnloadChunk::new(chunk.x, chunk.y))
                    .await;
            }
        }

        if !loading_chunks.is_empty() {
            entity.world.spawn_world_entity_chunks(
                player.clone(),
                loading_chunks,
                new_chunk_center,
            );
        }
    }
}
