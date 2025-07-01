use criterion::{Criterion, criterion_group, criterion_main};

use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use std::sync::Arc;
use temp_dir::TempDir;

use pumpkin_world::dimension::Dimension;
use pumpkin_world::generation::{Seed, get_world_gen};
use pumpkin_world::level::Level;
use pumpkin_world::world::{BlockAccessor, BlockRegistryExt};

use tokio::runtime::Runtime;

struct BlockRegistry;

#[async_trait]
impl BlockRegistryExt for BlockRegistry {
    async fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _block_accessor: &dyn BlockAccessor,
        _block_pos: &BlockPos,
        _face: BlockDirection,
    ) -> bool {
        true
    }
}

async fn chunk_generation_seed(seed: i64) {
    let generator = get_world_gen(Seed(seed as u64), Dimension::Overworld);
    let temp_dir = TempDir::new().unwrap();
    let block_registry = Arc::new(BlockRegistry);
    let level = Arc::new(Level::from_root_folder(
        temp_dir.path().to_path_buf(),
        block_registry.clone(),
        seed,
        Dimension::Overworld,
    ));
    let x = 0;
    let y = 0;
    let position = Vector2::new(x, y);
    generator
        .generate_chunk(&level, block_registry.as_ref(), &position)
        .await;
}

fn bench_chunk_generation(c: &mut Criterion) {
    let seeds = [0, 42, 120, 200, 1000, 5000];
    let runtime = Runtime::new().unwrap();
    for seed in seeds {
        let name = format!("chunk generation seed {seed}");
        c.bench_function(&name, |b| {
            b.to_async(&runtime).iter(|| chunk_generation_seed(seed))
        });
    }
}

criterion_group!(benches, bench_chunk_generation);
criterion_main!(benches);
