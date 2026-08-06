//! Measures the server-side cost of putting one full overworld chunk on the wire.
//!
//! This is the number that sets the chunk-send ceiling. The vanilla 1.21.4 client sizes its
//! next batch request from the wall time it observes between `ClientboundChunkBatchStartPacket`
//! and `ClientboundChunkBatchFinishedPacket`:
//! `ChunkBatchSizeCalculator.getDesiredChunksPerTick()` returns `7000000.0 /
//! aggregatedNanosPerChunk` (net/minecraft/client/multiplayer/ChunkBatchSizeCalculator.java,
//! 1.21.4 Mojang-named source). Every microsecond the server spends inside a batch therefore
//! shrinks the number of chunks the client asks for on the next tick.
//!
//! `serialize` is `CChunkData::write_packet_data` alone.
//! `serialize_and_compress` adds the zlib pass the packet encoder performs above the
//! 256-byte default threshold at the default level 4.

#![allow(clippy::print_stdout, reason = "a benchmark reporting payload sizes")]

use criterion::{Criterion, criterion_group, criterion_main};
use flate2::{Compress, Compression, FlushCompress};
use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::ClientPacket;
use pumpkin_protocol::java::client::play::CChunkData;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk_system::{Chunk, StagedChunkEnum, generate_single_chunk};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::world::WorldPortalExt;
use std::hint::black_box;
use std::sync::Arc;

const SEED: Seed = Seed(42);

struct BlockRegistry;
impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

fn generated_chunk() -> Arc<ChunkData> {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD, false, Vec::new(), String::new());
    let registry = BlockRegistry;
    match generate_single_chunk(
        &Dimension::OVERWORLD,
        0,
        &world_gen,
        &registry,
        0,
        0,
        StagedChunkEnum::Full,
    ) {
        Chunk::Level(chunk) => chunk,
        Chunk::Proto(_) => panic!("Full stage must produce a level chunk"),
    }
}

fn bench_chunk_packet(c: &mut Criterion) {
    let chunk = generated_chunk();

    let mut raw = Vec::new();
    CChunkData(&chunk)
        .write_packet_data(&mut raw, &CURRENT_MC_VERSION)
        .expect("serialize");
    println!("uncompressed chunk packet: {} bytes", raw.len());

    c.bench_function("chunk_packet_serialize", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(raw.len());
            CChunkData(&chunk)
                .write_packet_data(&mut buf, &CURRENT_MC_VERSION)
                .expect("serialize");
            black_box(buf);
        });
    });

    let mut compressor = Compress::new(Compression::new(4), true);
    let mut scratch = Vec::with_capacity(raw.len());
    compressor
        .compress_vec(&raw, &mut scratch, FlushCompress::Finish)
        .expect("compress");
    println!("compressed (level 4): {} bytes", scratch.len());

    c.bench_function("chunk_packet_serialize_and_compress", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(raw.len());
            CChunkData(&chunk)
                .write_packet_data(&mut buf, &CURRENT_MC_VERSION)
                .expect("serialize");
            let mut compressor = Compress::new(Compression::new(4), true);
            let mut out = Vec::with_capacity(buf.len());
            compressor
                .compress_vec(&buf, &mut out, FlushCompress::Finish)
                .expect("compress");
            black_box(out);
        });
    });
}

criterion_group!(benches, bench_chunk_packet);
criterion_main!(benches);
