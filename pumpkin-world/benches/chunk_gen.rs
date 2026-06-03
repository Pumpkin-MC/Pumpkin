use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::ProtoChunk;
use pumpkin_world::chunk_system::{Cache, Chunk, StagedChunkEnum, generate_single_chunk};
use pumpkin_world::generation::generator::VanillaGenerator;
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::world::WorldPortalExt;
use std::hint::black_box;
use std::sync::Arc;

const SEED: Seed = Seed(42);

// Stub portal — allows all block placements, skips mob spawning.
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

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

/// Build a cache containing chunk 0, 0 at the center so that features
/// and lighting stages get a 3×3 cache.
fn setup_cache(
    target_stage: StagedChunkEnum,
    world_gen: &VanillaGenerator,
    block_registry: &dyn WorldPortalExt,
) -> Cache {
    let radius = target_stage.get_direct_radius();
    let mut cache = Cache::new(-radius, -radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            cache
                .chunks
                .push(Chunk::Proto(Box::new(ProtoChunk::new(dx, dz, world_gen))));
        }
    }

    let pipeline = [
        StagedChunkEnum::Biomes,
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Carvers,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
    ];
    for stage in pipeline {
        if stage as u8 >= target_stage as u8 {
            break;
        }
        cache.advance(
            stage,
            world_gen,
            block_registry,
            &LightingEngineConfig::Default,
        );
    }

    cache
}

/// Full pipeline

fn bench_full_chunk_generation(c: &mut Criterion) {
    let dimension = Dimension::OVERWORLD;
    let world_gen = get_world_gen(SEED, dimension.clone());
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("full_chunk_generation", |b| {
        b.iter(|| {
            black_box(generate_single_chunk(
                &dimension,
                0,
                &world_gen,
                block_registry.as_ref(),
                black_box(0),
                black_box(0),
                StagedChunkEnum::Full,
            ));
        });
    });
}

/// Per-phase benchmarks

fn bench_biomes_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("biomes_generation", |b| {
        b.iter_batched(
            || setup_cache(StagedChunkEnum::Biomes, &world_gen, block_registry.as_ref()),
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Biomes,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_structure_starts_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("structure_starts_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::StructureStart,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::StructureStart,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_structure_references_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("structure_references_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::StructureReferences,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::StructureReferences,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_noise_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("noise_generation", |b| {
        b.iter_batched(
            || setup_cache(StagedChunkEnum::Noise, &world_gen, block_registry.as_ref()),
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Noise,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_surface_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("surface_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::Surface,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Surface,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_carvers_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("carvers_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::Carvers,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Carvers,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_features_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("features_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::Features,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Features,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_lighting_generation(c: &mut Criterion) {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD);
    let block_registry = Arc::new(BlockRegistry);

    c.bench_function("lighting_generation", |b| {
        b.iter_batched(
            || {
                setup_cache(
                    StagedChunkEnum::Lighting,
                    &world_gen,
                    block_registry.as_ref(),
                )
            },
            |mut cache| {
                cache.advance(
                    StagedChunkEnum::Lighting,
                    &world_gen,
                    block_registry.as_ref(),
                    &LightingEngineConfig::Default,
                );
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_full_chunk_generation,
    bench_biomes_generation,
    bench_structure_starts_generation,
    bench_structure_references_generation,
    bench_noise_generation,
    bench_surface_generation,
    bench_carvers_generation,
    bench_features_generation,
    bench_lighting_generation,
);
criterion_main!(benches);
