use criterion::{Criterion, criterion_group, criterion_main};

use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::generation::implementation::WorldGenerator;
use std::sync::Arc;
use temp_dir::TempDir;

use pumpkin_world::dimension::Dimension;
use pumpkin_world::generation::{Seed, get_world_gen};
use pumpkin_world::level::Level;
use pumpkin_world::world::{BlockAccessor, BlockRegistryExt};

use rayon::prelude::*;

struct BlockRegistry;

#[async_trait]
impl BlockRegistryExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _block_accessor: &dyn BlockAccessor,
        _block_pos: &BlockPos,
        _face: BlockDirection,
    ) -> bool {
        true
    }
}

fn chunk_generation_seed(seed: i64) {
    let generator: Arc<dyn WorldGenerator> =
        get_world_gen(Seed(seed as u64), Dimension::Overworld).into();
    let temp_dir = TempDir::new().unwrap();
    let block_registry = Arc::new(BlockRegistry);
    let level = Arc::new(Level::from_root_folder(
        temp_dir.path().to_path_buf(),
        block_registry.clone(),
        seed,
        Dimension::Overworld,
    ));

    // Prepare all positions to generate
    let positions: Vec<Vector2<i32>> = (0..100)
        .flat_map(|x| (0..10).map(move |y| Vector2::new(x, y)))
        .collect();

    positions.par_iter().for_each(|position| {
        generator.generate_chunk(&level, block_registry.as_ref(), position);
    });
}

fn bench_chunk_generation(c: &mut Criterion) {
    let seeds = [0];
    for seed in seeds {
        let name = format!("chunk generation seed {seed}");
        c.bench_function(&name, |b| b.iter(|| chunk_generation_seed(seed)));
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::from_secs(180));
    targets = bench_chunk_generation
}
criterion_main!(benches);
