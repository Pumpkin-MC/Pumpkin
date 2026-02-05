use super::chunk_state::{Chunk, StagedChunkEnum};
use super::generation_cache::Cache;
use super::{ChunkPos, IOLock};
use crate::ProtoChunk;
use crate::chunk::io::LoadedData;
use crate::chunk::io::LoadedData::Loaded;
use crate::chunk::format::LightContainer;
use crate::level::Level;
use crossfire::compat::AsyncRx;
use itertools::Itertools;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::chunk::ChunkStatus;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::thread;
use pumpkin_data::chunk_gen_settings::GenerationSettings;

pub enum RecvChunk {
    IO(Chunk),
    Generation(Cache),
}

/// Checks if a chunk needs relighting based on the current lighting configuration
/// Returns true if the chunk has uniform lighting (from full/dark mode) but the server
/// is now running in default mode (which needs proper lighting calculation)
fn needs_relighting(chunk: &crate::chunk::ChunkData, config: &LightingEngineConfig) -> bool {
    // Only need relighting if we're in default mode
    if *config != LightingEngineConfig::Default {
        return false;
    }
    
    // Check if all light sections are uniformly filled (all 0xFF) or empty (all 0x00)
    // This indicates the chunk was generated/saved with full or dark mode
    let all_sky_uniform = chunk.light_engine.sky_light.iter().all(|lc| {
        match lc {
            LightContainer::Full(data) => data.iter().all(|b| *b == 0xFF || *b == 0x00),
            LightContainer::Empty(0) | LightContainer::Empty(15) => true,
            _ => false,
        }
    });
    
    let all_block_uniform = chunk.light_engine.block_light.iter().all(|lc| {
        match lc {
            LightContainer::Full(data) => data.iter().all(|b| *b == 0xFF || *b == 0x00),
            LightContainer::Empty(0) | LightContainer::Empty(15) => true,
            _ => false,
        }
    });
    
    all_sky_uniform && all_block_uniform
}

pub async fn io_read_work(
    recv: crossfire::compat::MAsyncRx<ChunkPos>,
    send: crossfire::compat::MTx<(ChunkPos, RecvChunk)>,
    level: Arc<Level>,
    lock: IOLock,
) {
    use crate::biome::hash_seed;
    log::debug!("io read thread start");
    let biome_mixer_seed = hash_seed(level.world_gen.random_config.seed);
    let dimension = &level.world_gen.dimension;
    let (t_send, mut t_recv) = tokio::sync::mpsc::channel(1);
    loop {
        let pos = match tokio::select! {
            biased;
            _ = level.cancel_token.cancelled() => None,
            res = recv.recv() => match res {
                Ok(p) => Some(p),
                Err(_) => None,
            }
        } {
            Some(p) => p,
            None => break,
        };

        tokio::task::block_in_place(|| {
            let mut data = lock.0.lock().unwrap();
            while data.contains_key(&pos) {
                data = lock.1.wait(data).unwrap();
            }
        });
        level
            .chunk_saver
            .fetch_chunks(&level.level_folder, &[pos], t_send.clone())
            .await;
        let data = match tokio::select! {
            biased;
            _ = level.cancel_token.cancelled() => None,
            res = t_recv.recv() => res,
        } {
            Some(res) => res,
            None => break,
        };
        match data {
            Loaded(chunk) => {
                if chunk.read().await.status == ChunkStatus::Full {
                    let read_guard = chunk.read().await;
                    let needs_relight = needs_relighting(&read_guard, &level.lighting_config);
                    
                    if needs_relight {
                        // Chunk has uniform lighting but server is in default mode
                        // Need to run through Lighting stage to recalculate lighting
                        log::debug!("Chunk {pos:?} has uniform lighting, downgrading to Features stage for relighting");
                        let mut proto = ProtoChunk::from_chunk_data(
                            &*read_guard,
                            dimension,
                            level.world_gen.default_block,
                            biome_mixer_seed,
                        );
                        
                        // Clear all lighting data before recalculation
                        // The lighting engine expects to start from 0 (dark)
                        // If we don't clear it, the old lighting will propagate incorrectly.
                        let section_count = proto.light.sky_light.len();
                        proto.light.sky_light = (0..section_count)
                            .map(|_| {
                                if dimension.has_skylight {
                                    LightContainer::new_filled(0)
                                } else {
                                    LightContainer::new_empty(0)
                                }
                            })
                            .collect();
                        proto.light.block_light = (0..section_count)
                            .map(|_| LightContainer::new_filled(0))
                            .collect();
                        
                        // Set stage to Features so scheduler will run Lighting -> Full
                        proto.stage = StagedChunkEnum::Features;
                        drop(read_guard);
                        
                        if send.send((pos, RecvChunk::IO(Chunk::Proto(Box::new(proto))))).is_err() {
                            break;
                        }
                    } else {
                        // Chunk is fully generated with proper lighting, send it as-is
                        drop(read_guard);
                        if send.send((pos, RecvChunk::IO(Chunk::Level(chunk)))).is_err() {
                            break;
                        }
                    }
                } else {
                    // debug!("io read thread receive proto chunk {pos:?}",);
                    let val = RecvChunk::IO(Chunk::Proto(Box::new(ProtoChunk::from_chunk_data(
                        &*chunk.read().await,
                        dimension,
                        level.world_gen.default_block,
                        biome_mixer_seed,
                    ))));
                    if send.send((pos, val)).is_err() {
                        break;
                    }
                }
                continue;
            }
            LoadedData::Missing(_) => {}
            LoadedData::Error(_) => {
                log::warn!("chunk data read error pos: {pos:?}. regenerating");
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
            .is_err()
        {
            break;
        }
    }
    log::debug!("io read thread stop");
}

pub async fn io_write_work(recv: AsyncRx<Vec<(ChunkPos, Chunk)>>, level: Arc<Level>, lock: IOLock) {
    log::info!("io write thread start",);
    loop {
        // Don't check cancel_token here (keep saving chunks)
        let data = match recv.recv().await {
            Ok(d) => d,
            Err(_) => {
                log::debug!("io write channel closed, exiting");
                break;
            }
        };
        // debug!("io write thread receive chunks size {}", data.len());
        let mut vec = Vec::with_capacity(data.len());
        for (pos, chunk) in data {
            match chunk {
                Chunk::Level(chunk) => vec.push((pos, chunk)),
                Chunk::Proto(chunk) => {
                    let mut temp = Chunk::Proto(chunk);
                    temp.upgrade_to_level_chunk(&level.world_gen.dimension);
                    let Chunk::Level(chunk) = temp else { panic!() };
                    vec.push((pos, chunk));
                }
            }
        }
        let pos = vec.iter().map(|(pos, _)| *pos).collect_vec();
        level
            .chunk_saver
            .save_chunks(&level.level_folder, vec)
            .await
            .unwrap();
        for i in pos {
            let mut data = lock.0.lock().unwrap();
            match data.entry(i) {
                Entry::Occupied(mut entry) => {
                    let rc = entry.get_mut();
                    if *rc == 1 {
                        entry.remove();
                        drop(data);
                        lock.1.notify_all();
                    } else {
                        *rc -= 1;
                    }
                }
                Entry::Vacant(_) => panic!(),
            }
        }
    }
    log::info!(
        "io write thread stop id: {:?} name: {}",
        thread::current().id(),
        thread::current().name().unwrap_or("unknown")
    );
}

pub fn generation_work(
    recv: crossfire::compat::MRx<(ChunkPos, Cache, StagedChunkEnum)>,
    send: crossfire::compat::MTx<(ChunkPos, RecvChunk)>,
    level: Arc<Level>,
) {
    log::debug!(
        "generation thread start id: {:?} name: {}",
        thread::current().id(),
        thread::current().name().unwrap_or("unknown")
    );

    let settings = GenerationSettings::from_dimension(&level.world_gen.dimension);
    use std::time::Duration;
    loop {
        match recv.try_recv() {
            Ok((pos, mut cache, stage)) => {
                // debug!("generation thread receive chunk pos {pos:?} to stage {stage:?}");
                cache.advance(
                    stage,
                    level.block_registry.as_ref(),
                    settings,
                    &level.world_gen.random_config,
                    &level.world_gen.terrain_cache,
                    &level.world_gen.base_router,
                    level.world_gen.dimension,
                );
                if send.send((pos, RecvChunk::Generation(cache))).is_err() {
                    break;
                }
            }
            Err(_) => {
                if level.cancel_token.is_cancelled() {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        }
    }
    log::debug!(
        "generation thread stop id: {:?} name: {}",
        thread::current().id(),
        thread::current().name().unwrap_or("unknown")
    );
}
