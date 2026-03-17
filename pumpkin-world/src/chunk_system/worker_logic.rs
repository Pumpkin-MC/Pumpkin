use super::chunk_state::{Chunk, StagedChunkEnum};
use super::generation_cache::Cache;
use super::{ChunkPos, IOLock};
use crate::ProtoChunk;
use crate::chunk::format::LightContainer;
use crate::chunk::io::LoadedData;
use crate::chunk::io::LoadedData::Loaded;
use crate::level::Level;
use crossfire::compat::AsyncRx;
use itertools::Itertools;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use tracing::{debug, error, warn};

pub enum RecvChunk {
    IO(Chunk),
    Generation(Cache),
    GenerationFailure {
        pos: ChunkPos,
        stage: StagedChunkEnum,
        error: String,
        cache: Option<Cache>,
    },
}

/// Checks if a chunk needs relighting based on the current lighting configuration
/// Returns true if the chunk has uniform lighting (from full/dark mode) but the server
/// is now running in default mode (which needs proper lighting calculation)
fn needs_relighting(chunk: &crate::chunk::ChunkData, config: &LightingEngineConfig) -> bool {
    if *config != LightingEngineConfig::Default {
        return false;
    }

    // If the chunk says it's already lit, believe it.
    if chunk.light_populated.load(Relaxed) {
        return false;
    }

    let engine = chunk.light_engine.lock().expect("Mutex poisoned");

    // Scan for any complex lighting data
    let has_complex_light = engine.sky_light.iter().any(|lc| match lc {
        LightContainer::Full(data) => data.iter().any(|&b| b != 0x00 && b != 0xFF),
        LightContainer::Empty(val) => *val != 0 && *val != 15,
    }) || engine.block_light.iter().any(|lc| match lc {
        LightContainer::Full(data) => data.iter().any(|&b| b != 0x00 && b != 0xFF),
        LightContainer::Empty(val) => *val != 0 && *val != 15,
    });

    // If it has complex light, we don't need to relight.
    !has_complex_light
}

pub async fn io_read_work(
    recv: crossfire::compat::MAsyncRx<ChunkPos>,
    send: crossfire::compat::MAsyncTx<(ChunkPos, RecvChunk)>,
    level: Arc<Level>,
    lock: IOLock,
) {
    use crate::biome::hash_seed;
    debug!("io read thread start");
    let biome_mixer_seed = hash_seed(level.world_gen.random_config.seed);
    let dimension = &level.world_gen.dimension;

    while let Ok(pos) = recv.recv().await {
        // Asynchronously wait for the lock using Tokio's Notify and Timeout
        let lock_acquired = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                // 1. Get the future BEFORE checking the condition to avoid race conditions
                let notified = lock.1.notified();

                // 2. Briefly lock the Mutex to check the state
                {
                    let data = lock.0.lock().unwrap();
                    if !data.contains_key(&pos) {
                        break; // Lock conceptually acquired, proceed
                    }
                }

                // 3. Await the notification if the chunk was still locked
                notified.await;
            }
        })
        .await
        .is_ok();

        if !lock_acquired {
            warn!("io_read: timed out waiting for IO write lock on chunk {pos:?}, re-queuing");

            // Fix: Added .await here
            if send
                .send((
                    pos,
                    RecvChunk::GenerationFailure {
                        pos,
                        stage: StagedChunkEnum::Empty,
                        error: "IO write lock timeout".to_string(),
                        cache: None,
                    },
                ))
                .await
                .is_err()
            {
                break;
            }
            continue;
        }

        // Create the channel inside the loop!
        let (t_send, mut t_recv) = tokio::sync::mpsc::channel(1);

        level
            .chunk_saver
            .fetch_chunks(&level.level_folder, &[pos], t_send)
            .await;

        let data = match t_recv.recv().await {
            Some(res) => res,
            None => break,
        };

        match data {
            Loaded(chunk) => {
                if chunk.status == ChunkStatus::Full {
                    let needs_relight = needs_relighting(&chunk, &level.lighting_config);

                    if needs_relight {
                        debug!(
                            "Chunk {pos:?} has uniform lighting, downgrading to Features stage for relighting"
                        );

                        let mut proto = ProtoChunk::from_chunk_data(
                            &chunk,
                            dimension,
                            level.world_gen.default_block,
                            biome_mixer_seed,
                        );

                        let section_count = proto.light.sky_light.len();
                        proto.light.sky_light = (0..section_count)
                            .map(|_| LightContainer::new_empty(0))
                            .collect();
                        proto.light.block_light = (0..section_count)
                            .map(|_| LightContainer::new_empty(0))
                            .collect();

                        proto.stage = StagedChunkEnum::Features;

                        if send
                            .send((pos, RecvChunk::IO(Chunk::Proto(Box::new(proto)))))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    } else {
                        if send
                            .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                } else {
                    let val = RecvChunk::IO(Chunk::Proto(Box::new(ProtoChunk::from_chunk_data(
                        &chunk,
                        dimension,
                        level.world_gen.default_block,
                        biome_mixer_seed,
                    ))));
                    if send.send((pos, val)).await.is_err() {
                        break;
                    }
                }
                continue;
            }
            LoadedData::Missing(_) => {}
            LoadedData::Error(_) => {
                warn!("chunk data read error pos: {pos:?}. regenerating");
            }
        }

        if send
            .send((
                pos,
                RecvChunk::IO(Chunk::Proto(Box::new(ProtoChunk::new(
                    pos.x,
                    pos.y,
                    dimension,
                    level.world_gen.default_block,
                    biome_mixer_seed,
                )))),
            ))
            .await
            .is_err()
        {
            break;
        }
    }
    debug!("io read thread stop");
}

pub async fn io_write_work(recv: AsyncRx<Vec<(ChunkPos, Chunk)>>, level: Arc<Level>, lock: IOLock) {
    loop {
        let data = match recv.recv().await {
            Ok(d) => d,
            Err(_) => break,
        };

        let mut vec = Vec::with_capacity(data.len());
        for (pos, chunk) in data {
            match chunk {
                Chunk::Level(chunk) => vec.push((pos, chunk)),
                Chunk::Proto(chunk) => {
                    let mut temp = Chunk::Proto(chunk);
                    temp.upgrade_to_level_chunk(&level.world_gen.dimension, &level.lighting_config);
                    let Chunk::Level(chunk) = temp else { panic!() };
                    vec.push((pos, chunk));
                }
            }
        }

        let pos = vec.iter().map(|(pos, _)| *pos).collect_vec();
        if let Err(e) = level
            .chunk_saver
            .save_chunks(&level.level_folder, vec)
            .await
        {
            error!("Failed to save chunks: {:?}", e);
        }

        for i in pos {
            let mut data = lock.0.lock().unwrap();
            match data.entry(i) {
                Entry::Occupied(mut entry) => {
                    let rc = entry.get_mut();
                    if *rc == 1 {
                        entry.remove();
                        drop(data); // Drop the lock BEFORE notifying
                        // Wake up all parked tasks checking the map
                        lock.1.notify_waiters();
                    } else {
                        *rc -= 1;
                    }
                }
                Entry::Vacant(_) => {
                    warn!(
                        "io_write: attempted to release missing lock entry for {:?}",
                        i
                    );
                }
            }
        }
    }
}
pub fn generation_work(
    recv: crossfire::compat::MRx<(ChunkPos, Cache, StagedChunkEnum)>,
    send: crossfire::compat::MTx<(ChunkPos, RecvChunk)>,
    level: Arc<Level>,
) {
    let settings = GenerationSettings::from_dimension(&level.world_gen.dimension);

    loop {
        let (pos, mut cache, stage) = if let Ok(data) = recv.recv() {
            data
        } else {
            debug!("generation channel closed, exiting");
            break;
        };

        // Run generation with panic catching
        let result = {
            let cache_ref = &mut cache;
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cache_ref.advance(
                    stage,
                    &level.lighting_config,
                    level.block_registry.as_ref(),
                    settings,
                    &level.world_gen.random_config,
                    &level.world_gen.terrain_cache,
                    &level.world_gen.base_router,
                    level.world_gen.dimension,
                );
            }))
        };

        match result {
            Ok(_) => {
                // If Ok, the borrow is over, and we can safely move `cache` into the success message.
                if send.send((pos, RecvChunk::Generation(cache))).is_err() {
                    break;
                }
            }
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| {
                        payload
                            .downcast_ref::<String>()
                            .map(std::string::String::as_str)
                    })
                    .unwrap_or("Unknown panic payload");

                error!("Chunk generation FAILED at {pos:?} ({stage:?}): {msg}");

                // Send failure notification
                let _ = send.send((
                    pos,
                    RecvChunk::GenerationFailure {
                        pos,
                        stage,
                        error: msg.to_string(),
                        cache: Some(cache),
                    },
                ));
            }
        }
    }
}
