use super::chunk_state::{Chunk, StagedChunkEnum};
use super::generation_cache::Cache;
use super::{ChunkPos, IOLock};
use crate::ProtoChunk;
use crate::chunk::format::LightContainer;
use crate::chunk::io::LoadedData;
use crate::chunk::io::LoadedData::Loaded;
use crate::level::Level;
use crossfire::compat::AsyncRx;
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
    recv: crossfire::compat::MAsyncRx<Vec<ChunkPos>>,
    send: crossfire::compat::MTx<(ChunkPos, RecvChunk)>,
    level: Arc<Level>,
    lock: IOLock,
) {
    debug!("io read thread start");

    // Cleaner loop and async recv
    while let Ok(batch) = recv.recv().await {
        for pos in &batch {
            // Lock handling
            loop {
                let notified = lock.1.notified();
                if !lock.0.lock().unwrap().contains_key(pos) {
                    break;
                }
                notified.await;
            }
        }

        let (t_send, mut t_recv) = tokio::sync::mpsc::channel(1000);

        let level_clone = level.clone();
        let batch_clone = batch.clone();

        let fetch_task = tokio::spawn(async move {
            level_clone
                .chunk_saver
                .fetch_chunks(&level_clone.level_folder, &batch_clone, t_send)
                .await;
        });

        for _ in 0..batch.len() {
            let data = match t_recv.recv().await {
                Some(res) => res,
                None => break,
            };

            match data {
                Loaded(chunk) => {
                    let pos = ChunkPos::new(chunk.x, chunk.z);
                    if chunk.status == ChunkStatus::Full {
                        // Relighting check
                        let needs_relight = needs_relighting(&chunk, &level.lighting_config);

                        if needs_relight {
                            debug!(
                                "Chunk {pos:?} has uniform lighting, downgrading to Features stage for relighting"
                            );

                            // Create ProtoChunk using the async method
                            let mut proto = ProtoChunk::from_chunk_data(&chunk, &level.world_gen);

                            // Clear all lighting data
                            let section_count = proto.light.sky_light.len();
                            proto.light.sky_light = (0..section_count)
                                .map(|_| LightContainer::new_empty(15))
                                .collect();
                            proto.light.block_light = (0..section_count)
                                .map(|_| LightContainer::new_empty(0))
                                .collect();

                            // Set stage to Features
                            proto.stage = StagedChunkEnum::Features;

                            if send
                                .send((pos, RecvChunk::IO(Chunk::Proto(Box::new(proto)))))
                                .is_err()
                            {
                                break;
                            }
                        } else {
                            // Send fully valid chunk
                            if send
                                .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                                .is_err()
                            {
                                break;
                            }
                        }
                    } else {
                        // Standard ProtoChunk handling for non-full chunks
                        let val = RecvChunk::IO(Chunk::Proto(Box::new(
                            ProtoChunk::from_chunk_data(&chunk, &level.world_gen),
                        )));
                        if send.send((pos, val)).is_err() {
                            break;
                        }
                    }
                }
                LoadedData::Missing(pos) | LoadedData::Error((pos, _)) => {
                    if send
                        .send((
                            pos,
                            RecvChunk::IO(Chunk::Proto(Box::new(ProtoChunk::new(
                                pos.x,
                                pos.y,
                                &level.world_gen,
                            )))),
                        ))
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
        let _ = fetch_task.await;
    }
    debug!("io read thread stop");
}

pub async fn io_write_work(recv: AsyncRx<Vec<(ChunkPos, Chunk)>>, level: Arc<Level>, lock: IOLock) {
    loop {
        // Don't check cancel_token here (keep saving chunks)
        let data = match recv.recv().await {
            Ok(d) => d,
            Err(_) => break,
        };
        // debug!("io write thread receive chunks size {}", data.len());
        let mut vec = Vec::with_capacity(data.len());
        let mut positions = Vec::with_capacity(data.len());
        for (pos, chunk) in data {
            positions.push(pos);

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
        if let Err(e) = level
            .chunk_saver
            .save_chunks(&level.level_folder, vec)
            .await
        {
            error!("Failed to save chunks: {:?}", e);
        }

        for i in positions {
            let mut data = lock.0.lock().unwrap();
            match data.entry(i) {
                Entry::Occupied(mut entry) => {
                    let rc = entry.get_mut();
                    if *rc == 1 {
                        entry.remove();
                        drop(data);
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
                    // continue without panicking to avoid crashing on shutdown races
                }
            }
        }
    }
}

pub fn run_generation(
    pos: ChunkPos,
    mut cache: Cache,
    stage: StagedChunkEnum,
    level: &Level,
    _settings: &GenerationSettings,
) -> RecvChunk {
    let portal = level.world_portal.load_full();
    let portal_ref = portal.as_deref().expect("Portal should be initialized");
    // Run generation with panic catching
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cache.advance(stage, &level.world_gen, portal_ref, &level.lighting_config);
        cache // Return cache on success
    }));

    match result {
        Ok(cache) => RecvChunk::Generation(cache),
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

            RecvChunk::GenerationFailure {
                pos,
                stage,
                error: msg.to_string(),
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
        let (pos, cache, stage) = if let Ok(data) = recv.recv() {
            data
        } else {
            debug!("generation channel closed, exiting");
            break;
        };

        let result = run_generation(pos, cache, stage, &level, settings);
        if send.send((pos, result)).is_err() {
            break;
        }
    }
}
