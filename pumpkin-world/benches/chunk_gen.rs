use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::biome::hash_seed;
use pumpkin_world::chunk_system::{StagedChunkEnum, generate_single_chunk};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::world::BlockRegistryExt;
use std::hint::black_box;
use std::sync::Arc;

fn bench_full_chunk_generation(c: &mut Criterion) {
    let dimension = Dimension::OVERWORLD;
    let seed = Seed(42);
    let block_registry = Arc::new(BlockRegistry);
    let world_gen = get_world_gen(seed, dimension);
    let biome_mixer_seed = hash_seed(world_gen.random_config.seed);

    c.bench_function("full_chunk_generation", |b| {
        b.iter(|| {
            let chunk = generate_single_chunk(
                black_box(&dimension),
                black_box(biome_mixer_seed),
                black_box(&world_gen),
                black_box(block_registry.as_ref()),
                black_box(0),
                black_box(0),
                black_box(StagedChunkEnum::Full),
            );
            black_box(chunk);
        });
    });
}

struct BlockRegistry;
impl BlockRegistryExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        true
    }
}

criterion_group!(benches, bench_full_chunk_generation,);
criterion_main!(benches);
