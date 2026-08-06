use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::{BlockStateId, dimension::Dimension};
use pumpkin_util::{math::vector2::Vector2, world_seed::Seed};
use pumpkin_world::{
    chunk::{ChunkData, format::anvil::SingleChunkDataSerializer},
    chunk_system::{Chunk, StagedChunkEnum, generate_single_chunk},
    generation::get_world_gen,
    world::WorldPortalExt,
};
use std::hint::black_box;

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

fn bench_chunk_deserialization(c: &mut Criterion) {
    let dimension = Dimension::OVERWORLD;
    let world_gen = get_world_gen(
        Seed(42),
        dimension.clone(),
        false,
        Vec::new(),
        String::new(),
    );
    let chunk = generate_single_chunk(
        &dimension,
        0,
        &world_gen,
        &BlockRegistry,
        0,
        0,
        StagedChunkEnum::Full,
    );
    let Chunk::Level(chunk) = chunk else {
        panic!("full generation must return a level chunk");
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("failed to create benchmark runtime");
    let bytes = runtime
        .block_on(chunk.to_bytes())
        .expect("failed to serialize benchmark chunk");
    let position = Vector2::new(chunk.x, chunk.z);

    c.bench_function("chunk_nbt_deserialization", |b| {
        b.iter(|| {
            black_box(
                ChunkData::from_bytes(black_box(&bytes), position)
                    .expect("failed to deserialize benchmark chunk"),
            );
        });
    });
}

criterion_group!(benches, bench_chunk_deserialization);
criterion_main!(benches);
