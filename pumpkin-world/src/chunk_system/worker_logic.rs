use super::chunk_state::{Chunk, StagedChunkEnum};
use super::generation_cache::Cache;
use super::{ChunkPos, IOLock};
use crate::ProtoChunk;
use crate::chunk::io::LoadedData;
use crate::chunk::io::LoadedData::Loaded;
use crate::generation::settings::gen_settings_from_dimension;
use crate::level::Level;
use crossfire::compat::AsyncRx;
use itertools::Itertools;
use pumpkin_data::chunk::ChunkStatus;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::thread;

pub enum RecvChunk {
    IO(Chunk),
    Generation(Cache),
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
    while let Ok(pos) = recv.recv().await {
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
        let data = match t_recv.recv().await {
            Some(res) => res,
            None => break,
        };
        match data {
            Loaded(chunk) => {
                if chunk.read().await.status == ChunkStatus::Full {
                    if send
                        .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                        .is_err()
                    {
                        break;
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
    while let Ok(data) = recv.recv().await {
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

    let settings = gen_settings_from_dimension(&level.world_gen.dimension);
    while let Ok((pos, mut cache, stage)) = recv.recv() {
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
    log::debug!(
        "generation thread stop id: {:?} name: {}",
        thread::current().id(),
        thread::current().name().unwrap_or("unknown")
    );
}
